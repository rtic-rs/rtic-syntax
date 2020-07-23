mod app;
mod extern_interrupt;
mod hardware_task;
mod idle;
mod init;
mod late_resource;
mod local;
mod software_task;
mod util;

use proc_macro2::TokenStream as TokenStream2;
use syn::{
    braced, parenthesized,
    parse::{self, Parse, ParseStream, Parser},
    token::Brace,
    Ident, Item, LitInt, Token, TypeTuple,
};

use crate::{
    ast::{App, AppArgs, HardwareTaskArgs, InitArgs, Resources, SoftwareTaskArgs},
    Either, Set, Settings,
};

pub fn app(args: TokenStream2, input: TokenStream2, settings: &Settings) -> parse::Result<App> {
    let args = AppArgs::parse(args)?;
    let input: Input = syn::parse2(input)?;

    App::parse(args, input, settings)
}

pub(crate) struct Input {
    _const_token: Token![const],
    pub ident: Ident,
    _colon_token: Token![:],
    _ty: TypeTuple,
    _eq_token: Token![=],
    _brace_token: Brace,
    pub items: Vec<Item>,
    _semi_token: Token![;],
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        fn parse_items(input: ParseStream<'_>) -> parse::Result<Vec<Item>> {
            let mut items = vec![];

            while !input.is_empty() {
                items.push(input.parse()?);
            }

            Ok(items)
        }

        let content;
        Ok(Input {
            _const_token: input.parse()?,
            ident: input.parse()?,
            _colon_token: input.parse()?,
            _ty: input.parse()?,
            _eq_token: input.parse()?,
            _brace_token: braced!(content in input),
            items: content.call(parse_items)?,
            _semi_token: input.parse()?,
        })
    }
}

fn init_idle_args(
    tokens: TokenStream2,
    settings: &Settings,
) -> parse::Result<InitArgs> {
    (|input: ParseStream<'_>| -> parse::Result<InitArgs> {
        if input.is_empty() {
            return Ok(InitArgs::default());
        }

        let mut late = None;
        let mut resources = None;
        let mut spawn = None;
        let mut schedule = None;

        let content;
        parenthesized!(content in input);
        loop {
            if content.is_empty() {
                break;
            }

            // #ident = ..
            let ident: Ident = content.parse()?;
            let _: Token![=] = content.parse()?;

            let ident_s = ident.to_string();
            match &*ident_s {
                /*
                "core" if cores != 1 => {
                    if core.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    let lit: LitInt = content.parse()?;
                    core = Some(util::parse_core(lit, cores)?);
                }
                */

                "late" => {
                    if late.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    let idents = util::parse_idents(&content)?;

                    late = Some(idents);
                }

                "resources" => {
                    if resources.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    resources = Some(util::parse_resources(&content)?);
                }

                "spawn" | "schedule" => {
                    let idents = util::parse_idents(&content)?;

                    let ident_s = ident.to_string();
                    match &*ident_s {
                        "spawn" => {
                            if spawn.is_some() {
                                return Err(parse::Error::new(
                                    ident.span(),
                                    "argument appears more than once",
                                ));
                            }

                            spawn = Some(idents);
                        }

                        "schedule" if settings.parse_schedule => {
                            if schedule.is_some() {
                                return Err(parse::Error::new(
                                    ident.span(),
                                    "argument appears more than once",
                                ));
                            }

                            schedule = Some(idents);
                        }

                        _ => {
                            return Err(parse::Error::new(ident.span(), "unexpected argument"));
                        }
                    }
                }

                _ => {
                    return Err(parse::Error::new(ident.span(), "unexpected argument"));
                }
            }

            if content.is_empty() {
                break;
            }

            // ,
            let _: Token![,] = content.parse()?;
        }

        Ok(InitArgs {
            core: 0,

            late: late.unwrap_or(Set::new()),

            resources: resources.unwrap_or(Resources::new()),
            spawn: spawn.unwrap_or(Set::new()),

            schedule: schedule.unwrap_or(Set::new()),

            _extensible: (),
        })
    })
    .parse2(tokens)
}

fn task_args(
    tokens: TokenStream2,
    settings: &Settings,
) -> parse::Result<Either<HardwareTaskArgs, SoftwareTaskArgs>> {
    (|input: ParseStream<'_>| -> parse::Result<Either<HardwareTaskArgs, SoftwareTaskArgs>> {
        if input.is_empty() {
            return Ok(Either::Right(SoftwareTaskArgs::default()));
        }

        let mut binds = None;
        let mut capacity = None;
        let mut priority = None;
        let mut resources = None;
        let mut schedule = None;
        let mut spawn = None;

        let content;
        parenthesized!(content in input);
        loop {
            if content.is_empty() {
                break;
            }

            // #ident = ..
            let ident: Ident = content.parse()?;
            let _: Token![=] = content.parse()?;

            let ident_s = ident.to_string();
            match &*ident_s {
                "binds" if settings.parse_binds => {
                    if binds.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    if capacity.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "hardware tasks can't use the `capacity` argument",
                        ));
                    }

                    // #ident
                    let ident = content.parse()?;

                    binds = Some(ident);
                }

                "capacity" => {
                    if capacity.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    if binds.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "hardware tasks can't use the `capacity` argument",
                        ));
                    }

                    // #lit
                    let lit: LitInt = content.parse()?;

                    if !lit.suffix().is_empty() {
                        return Err(parse::Error::new(
                            lit.span(),
                            "this literal must be unsuffixed",
                        ));
                    }

                    let value = lit.base10_parse::<u8>().ok();
                    if value.is_none() || value == Some(0) {
                        return Err(parse::Error::new(
                            lit.span(),
                            "this literal must be in the range 1...255",
                        ));
                    }

                    capacity = Some(value.unwrap());
                }

                /*
                "core" if cores != 1 => {
                    if core.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    let lit: LitInt = content.parse()?;
                    core = Some(util::parse_core(lit, cores)?);
                }
                */

                "priority" => {
                    if priority.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    // #lit
                    let lit: LitInt = content.parse()?;

                    if !lit.suffix().is_empty() {
                        return Err(parse::Error::new(
                            lit.span(),
                            "this literal must be unsuffixed",
                        ));
                    }

                    let value = lit.base10_parse::<u8>().ok();
                    if value.is_none() || value == Some(0) {
                        return Err(parse::Error::new(
                            lit.span(),
                            "this literal must be in the range 1...255",
                        ));
                    }

                    priority = Some(value.unwrap());
                }

                "resources" => {
                    if resources.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    resources = Some(util::parse_resources(&content)?);
                }

                "schedule" | "spawn" => {
                    if !settings.parse_schedule && ident_s == "schedule" {
                        return Err(parse::Error::new(ident.span(), "unexpected argument"));
                    }

                    // .. [#(#idents)*]
                    let idents = util::parse_idents(&content)?;
                    match &*ident_s {
                        "schedule" => {
                            if schedule.is_some() {
                                return Err(parse::Error::new(
                                    ident.span(),
                                    "argument appears more than once",
                                ));
                            }

                            schedule = Some(idents);
                        }

                        "spawn" => {
                            if spawn.is_some() {
                                return Err(parse::Error::new(
                                    ident.span(),
                                    "argument appears more than once",
                                ));
                            }

                            spawn = Some(idents);
                        }

                        _ => unreachable!(),
                    }
                }

                _ => {
                    return Err(parse::Error::new(ident.span(), "unexpected argument"));
                }
            }

            if content.is_empty() {
                break;
            }

            // ,
            let _: Token![,] = content.parse()?;
        }

        let core = 0;

        let priority = priority.unwrap_or(1);
        let resources = resources.unwrap_or(Resources::new());
        let schedule = schedule.unwrap_or(Set::new());
        let spawn = spawn.unwrap_or(Set::new());

        Ok(if let Some(binds) = binds {
            Either::Left(HardwareTaskArgs {
                core,
                binds,
                priority,
                resources,
                schedule,
                spawn,
                _extensible: (),
            })
        } else {
            Either::Right(SoftwareTaskArgs {
                core,
                capacity: capacity.unwrap_or(1),
                priority,
                resources,
                schedule,
                spawn,
                _extensible: (),
            })
        })
    })
    .parse2(tokens)
}
