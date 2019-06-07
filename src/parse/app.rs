use std::collections::{BTreeMap, HashSet};

use indexmap::map::Entry;
#[cfg(feature = "device")]
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{self, ParseStream, Parser},
    spanned::Spanned,
    ForeignItem, Ident, IntSuffix, Item, LitInt, Path, Token,
};

use super::Input;
use crate::{
    ast::{
        App, AppArgs, ExternInterrupt, HardwareTask, HardwareTaskArgs, HardwareTaskKind, Idle,
        IdleArgs, Init, InitArgs, LateResource, Resource, SoftwareTask, SoftwareTaskArgs,
    },
    parse::util,
    Map, Settings,
};

impl AppArgs {
    pub(crate) fn parse(tokens: TokenStream2, settings: &Settings) -> parse::Result<Self> {
        (|input: ParseStream<'_>| -> parse::Result<Self> {
            let mut cores = None;
            let mut device = None::<Path>;
            loop {
                if input.is_empty() {
                    break;
                }

                // #ident = ..
                let ident: Ident = input.parse()?;
                let _eq_token: Token![=] = input.parse()?;

                let ident_s = ident.to_string();
                match &*ident_s {
                    "cores" if settings.parse_cores => {
                        if cores.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        let lit = input.parse::<LitInt>()?;
                        if lit.suffix() != IntSuffix::None {
                            return Err(parse::Error::new(
                                lit.span(),
                                "this integer must be unsuffixed",
                            ));
                        }

                        let val = lit.value();
                        if val < 2 || val > u64::from(u8::max_value()) {
                            return Err(parse::Error::new(
                                lit.span(),
                                "number of cores must be in the range 2..=255",
                            ));
                        }

                        cores = Some(val as u8);
                    }

                    "device" if cfg!(feature = "device") => {
                        if device.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        device = Some(input.parse()?);
                    }

                    _ => {
                        return Err(parse::Error::new(ident.span(), "unexpected argument"));
                    }
                }

                if input.is_empty() {
                    break;
                }

                // ,
                let _: Token![,] = input.parse()?;
            }

            Ok(AppArgs {
                cores: cores.unwrap_or(1),

                #[cfg(feature = "device")]
                device: device.ok_or(parse::Error::new(
                    Span::call_site(),
                    "`device` argument is required",
                ))?,

                _extensible: (),
            })
        })
        .parse2(tokens)
    }
}

impl App {
    pub(crate) fn parse(args: AppArgs, input: Input, settings: &Settings) -> parse::Result<Self> {
        let cores = args.cores;

        let mut inits = BTreeMap::new();
        let mut idles = BTreeMap::new();

        let mut late_resources = Map::new();
        let mut resources = Map::new();
        let mut hardware_tasks = Map::new();
        let mut software_tasks = Map::new();

        let mut extern_interrupts = Map::new();

        let mut seen_idents = BTreeMap::<u8, HashSet<Ident>>::new();
        let mut check_ident = |core: u8, ident: &Ident| {
            let seen_idents = seen_idents.entry(core).or_default();

            if seen_idents.contains(ident) {
                return Err(parse::Error::new(
                    ident.span(),
                    if cores == 1 {
                        "this identifier has already been used"
                    } else {
                        "this identifier has already been used on this core"
                    },
                ));
            } else {
                seen_idents.insert(ident.clone());
            }

            Ok(())
        };
        for item in input.items {
            let mut is_exception = false;
            match item {
                Item::Fn(mut item) => {
                    let span = item.ident.span();
                    if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "init"))
                    {
                        let args =
                            InitArgs::parse(cores, item.attrs.remove(pos).tts, settings, span)?;

                        if inits.contains_key(&args.core) {
                            return Err(parse::Error::new(
                                span,
                                if cores == 1 {
                                    "`#[init]` function must appear at most once"
                                } else {
                                    "an `#[init]` function has already been assigned to this core"
                                },
                            ));
                        }

                        check_ident(args.core, &item.ident)?;

                        inits.insert(args.core, Init::parse(args, item)?);
                    } else if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "idle"))
                    {
                        let args =
                            IdleArgs::parse(cores, item.attrs.remove(pos).tts, settings, span)?;

                        if idles.contains_key(&args.core) {
                            return Err(parse::Error::new(
                                span,
                                if cores == 1 {
                                    "`#[idle]` function must appear at most once"
                                } else {
                                    "an `#[idle]` function has already been assigned to this core"
                                },
                            ));
                        }

                        check_ident(args.core, &item.ident)?;

                        idles.insert(args.core, Idle::parse(args, item)?);
                    } else if let Some(pos) = item.attrs.iter().position(|attr| {
                        if settings.parse_exception && util::attr_eq(attr, "exception") {
                            is_exception = true;
                            true
                        } else {
                            settings.parse_interrupt && util::attr_eq(attr, "interrupt")
                        }
                    }) {
                        if hardware_tasks.contains_key(&item.ident)
                            || software_tasks.contains_key(&item.ident)
                        {
                            return Err(parse::Error::new(
                                span,
                                "this task is defined multiple times",
                            ));
                        }

                        let args = HardwareTaskArgs::parse(
                            cores,
                            item.attrs.remove(pos).tts,
                            settings,
                            span,
                        )?;

                        check_ident(args.core, &item.ident)?;

                        hardware_tasks.insert(
                            item.ident.clone(),
                            HardwareTask::parse(
                                args,
                                if is_exception {
                                    HardwareTaskKind::Exception
                                } else {
                                    HardwareTaskKind::Interrupt
                                },
                                item,
                            )?,
                        );
                    } else if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "task"))
                    {
                        if hardware_tasks.contains_key(&item.ident)
                            || software_tasks.contains_key(&item.ident)
                        {
                            return Err(parse::Error::new(
                                span,
                                "this task is defined multiple times",
                            ));
                        }

                        let args = SoftwareTaskArgs::parse(
                            cores,
                            item.attrs.remove(pos).tts,
                            settings,
                            span,
                        )?;

                        check_ident(args.core, &item.ident)?;

                        software_tasks.insert(item.ident.clone(), SoftwareTask::parse(args, item)?);
                    } else {
                        return Err(parse::Error::new(
                            span,
                            "this item must live outside the `#[app]` module",
                        ));
                    }
                }

                Item::Static(item) => {
                    if late_resources.contains_key(&item.ident)
                        || resources.contains_key(&item.ident)
                    {
                        return Err(parse::Error::new(
                            item.ident.span(),
                            "this resource is listed more than once",
                        ));
                    }

                    resources.insert(item.ident.clone(), Resource::parse(item)?);
                }

                Item::ForeignMod(mod_) => {
                    if !util::abi_is_c(&mod_.abi) {
                        return Err(parse::Error::new(
                            mod_.abi.extern_token.span(),
                            "this `extern` block must use the \"C\" abi",
                        ));
                    }

                    for item in mod_.items {
                        match item {
                            ForeignItem::Fn(ref item) if settings.parse_extern_interrupt => {
                                match extern_interrupts.entry(item.ident.clone()) {
                                    Entry::Occupied(..) => {
                                        return Err(parse::Error::new(
                                            item.ident.span(),
                                            "this extern interrupt is listed more than once",
                                        ));
                                    }

                                    Entry::Vacant(entry) => {
                                        entry.insert(ExternInterrupt::parse(item, cores)?);
                                    }
                                }
                            }

                            ForeignItem::Static(item) => {
                                if late_resources.contains_key(&item.ident)
                                    || resources.contains_key(&item.ident)
                                {
                                    return Err(parse::Error::new(
                                        item.ident.span(),
                                        "this resource is listed more than once",
                                    ));
                                }

                                late_resources
                                    .insert(item.ident.clone(), LateResource::parse(item)?);
                            }

                            _ => {}
                        }
                    }
                }

                _ => {
                    return Err(parse::Error::new(
                        item.span(),
                        "this item must live outside the `#[app]` module",
                    ));
                }
            }
        }

        Ok(App {
            args,

            name: input.ident,

            inits,
            idles,

            late_resources,
            resources,
            hardware_tasks,
            software_tasks,

            extern_interrupts,

            _extensible: (),
        })
    }
}
