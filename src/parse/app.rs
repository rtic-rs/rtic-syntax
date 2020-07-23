use std::collections::{BTreeMap, HashSet};

use indexmap::map::Entry;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{self, ParseStream, Parser},
    spanned::Spanned,
    ExprParen, Fields, ForeignItem, Ident, Item, Lit, Path, Token, Visibility,
};

use super::Input;
use crate::{
    ast::{
        App, AppArgs, CustomArg, ExternInterrupt, ExternInterrupts, HardwareTask, Idle, IdleArgs,
        Init, InitArgs, LateResource, Resource, SoftwareTask,
    },
    parse::util,
    Either, Map, Settings,
};

impl AppArgs {
    pub(crate) fn parse(tokens: TokenStream2) -> parse::Result<Self> {
        (|input: ParseStream<'_>| -> parse::Result<Self> {
            //let mut cores = None;
            let mut custom = Map::new();

            loop {
                if input.is_empty() {
                    break;
                }

                // #ident = ..
                let ident: Ident = input.parse()?;
                let _eq_token: Token![=] = input.parse()?;

                let ident_s = ident.to_string();
                match &*ident_s {
                    /*
                    "cores" if settings.parse_cores => {
                        if cores.is_some() {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        let lit = input.parse::<LitInt>()?;
                        if !lit.suffix().is_empty() {
                            return Err(parse::Error::new(
                                lit.span(),
                                "this integer must be unsuffixed",
                            ));
                        }

                        let val = lit.base10_parse::<u8>().ok();
                        if val.is_none() || val.unwrap() < 2 {
                            return Err(parse::Error::new(
                                lit.span(),
                                "number of cores must be in the range 2..=255",
                            ));
                        }

                        cores = Some(val.unwrap());
                    }
                    */

                    _ => {
                        if custom.contains_key(&ident) {
                            return Err(parse::Error::new(
                                ident.span(),
                                "argument appears more than once",
                            ));
                        }

                        // Parse as path
                        if let Ok(p) = input.parse::<Path>() {
                            custom.insert(ident, CustomArg::Path(p));
                        } else {
                            // Parse as literal
                            match input.parse::<Lit>()? {
                                Lit::Bool(lit) => {
                                    custom.insert(ident, CustomArg::Bool(lit.value));
                                }
                                Lit::Int(lit) => {
                                    if lit.suffix().is_empty() {
                                        custom.insert(
                                            ident,
                                            CustomArg::UInt(lit.base10_digits().to_string()),
                                        );
                                    } else {
                                        return Err(parse::Error::new(
                                            ident.span(),
                                            "integer must be unsuffixed",
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(parse::Error::new(
                                        ident.span(),
                                        "argument has unexpected value",
                                    ));
                                }
                            }
                        }
                    }
                }

                if input.is_empty() {
                    break;
                }

                // ,
                let _: Token![,] = input.parse()?;
            }

            Ok(AppArgs {
                cores: 1,

                custom,
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

        let mut extern_interrupts = ExternInterrupts::new();

        let mut seen_idents = BTreeMap::<u8, HashSet<Ident>>::new();
        let mut bindings = BTreeMap::<u8, HashSet<Ident>>::new();
        let mut check_binding = |core: u8, ident: &Ident| {
            let bindings = bindings.entry(core).or_default();

            if bindings.contains(ident) {
                return Err(parse::Error::new(
                    ident.span(),
                    "a task has already been bound to this interrupt"
                ));
            } else {
                bindings.insert(ident.clone());
            }

            Ok(())
        };
        let mut check_ident = |core: u8, ident: &Ident| {
            let seen_idents = seen_idents.entry(core).or_default();

            if seen_idents.contains(ident) {
                return Err(parse::Error::new(
                    ident.span(),
                    "this identifier has already been used"
                ));
            } else {
                seen_idents.insert(ident.clone());
            }

            Ok(())
        };
        for mut item in input.items {
            match item {
                Item::Fn(mut item) => {
                    let span = item.sig.ident.span();
                    if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "init"))
                    {
                        let args =
                            InitArgs::parse(item.attrs.remove(pos).tokens, settings)?;

                        if inits.contains_key(&args.core) {
                            return Err(parse::Error::new(
                                span,
                                "`#[init]` function must appear at most once"
                            ));
                        }

                        check_ident(args.core, &item.sig.ident)?;

                        inits.insert(args.core, Init::parse(args, item)?);
                    } else if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "idle"))
                    {
                        let args =
                            IdleArgs::parse(item.attrs.remove(pos).tokens, settings)?;

                        if idles.contains_key(&args.core) {
                            return Err(parse::Error::new(
                                span,
                                "`#[idle]` function must appear at most once"
                            ));
                        }

                        check_ident(args.core, &item.sig.ident)?;

                        idles.insert(args.core, Idle::parse(args, item)?);
                    } else if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "task"))
                    {
                        if hardware_tasks.contains_key(&item.sig.ident)
                            || software_tasks.contains_key(&item.sig.ident)
                        {
                            return Err(parse::Error::new(
                                span,
                                "this task is defined multiple times",
                            ));
                        }

                        match crate::parse::task_args(
                            item.attrs.remove(pos).tokens,
                            //cores,
                            settings,
                            //span,
                        )? {
                            Either::Left(args) => {
                                check_binding(args.core, &args.binds)?;
                                check_ident(args.core, &item.sig.ident)?;

                                hardware_tasks.insert(
                                    item.sig.ident.clone(),
                                    HardwareTask::parse(args, item)?,
                                );
                            }

                            Either::Right(args) => {
                                check_ident(args.core, &item.sig.ident)?;

                                software_tasks.insert(
                                    item.sig.ident.clone(),
                                    SoftwareTask::parse(args, item)?,
                                );
                            }
                        }
                    } else {
                        return Err(parse::Error::new(
                            span,
                            "this item must live outside the `#[app]` module",
                        ));
                    }
                }

                Item::Struct(ref mut item) if item.ident.to_string() == "Resources" => {
                    if item.vis != Visibility::Inherited {
                        return Err(parse::Error::new(
                            item.span(),
                            "this item must have inherited / private visibility",
                        ));
                    }

                    if !item.attrs.is_empty() {
                        return Err(parse::Error::new(
                            item.span(),
                            "this item must have no attributes",
                        ));
                    }

                    if let Fields::Named(fields) = &mut item.fields {
                        for field in &mut fields.named {
                            let ident = field.ident.as_ref().expect("UNREACHABLE");

                            if late_resources.contains_key(ident) || resources.contains_key(ident) {
                                return Err(parse::Error::new(
                                    ident.span(),
                                    "this resource is listed more than once",
                                ));
                            }

                            if let Some(pos) = field
                                .attrs
                                .iter()
                                .position(|attr| util::attr_eq(attr, "init"))
                            {
                                let attr = field.attrs.remove(pos);

                                let late = LateResource::parse(field, ident.span())?;

                                resources.insert(
                                    ident.clone(),
                                    Resource {
                                        late,
                                        expr: syn::parse2::<ExprParen>(attr.tokens)?.expr,
                                    },
                                );
                            } else {
                                let late = LateResource::parse(field, ident.span())?;

                                late_resources.insert(ident.clone(), late);
                            }
                        }
                    } else {
                        return Err(parse::Error::new(
                            item.span(),
                            "this `struct` must have named fields",
                        ));
                    }
                }

                Item::ForeignMod(mod_) => {
                    if !util::abi_is_c(&mod_.abi) {
                        return Err(parse::Error::new(
                            mod_.abi.extern_token.span(),
                            "this `extern` block must use the \"C\" abi",
                        ));
                    }

                    for item in mod_.items {
                        if let ForeignItem::Fn(item) = item {
                            if settings.parse_extern_interrupt {
                                let (core, ident, extern_interrupt) =
                                    ExternInterrupt::parse(item, cores)?;

                                let extern_interrupts = extern_interrupts.entry(core).or_default();

                                let span = ident.span();
                                match extern_interrupts.entry(ident) {
                                    Entry::Occupied(..) => {
                                        return Err(parse::Error::new(
                                            span,
                                            "this extern interrupt is listed more than once"
                                        ));
                                    }

                                    Entry::Vacant(entry) => {
                                        entry.insert(extern_interrupt);
                                    }
                                }
                            } else {
                                return Err(parse::Error::new(
                                    item.sig.ident.span(),
                                    "this item must live outside the `#[app]` module",
                                ));
                            }
                        } else {
                            return Err(parse::Error::new(
                                item.span(),
                                "this item must live outside the `#[app]` module",
                            ));
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

#[cfg(test)]
mod tests {
    use crate::{ast::AppArgs, ast::CustomArg};

    #[test]
    fn parse_app_args_cores1() {
        let s = "peripherals = true";

        let stream: proc_macro2::TokenStream = s.parse().unwrap();
        let result = AppArgs::parse(stream).unwrap();

        // Check cores
        assert_eq!(result.cores, 1);

        // Check map
        for (ident, value) in result.custom {
            match ident.to_string().as_ref() {
                "peripherals" => match value {
                    CustomArg::Bool(true) => {}
                    _ => panic!("Expected peripherals = true"),
                },
                _ => panic!("Unexpected identifier"),
            }
        }
    }
}
