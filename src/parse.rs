mod app;
mod hardware_task;
mod idle;
mod init;
mod late_resource;
mod local;
mod monotonic;
mod software_task;
mod util;

use proc_macro2::TokenStream as TokenStream2;
use syn::{
    braced, parenthesized,
    parse::{self, Parse, ParseStream, Parser},
    token::Brace,
    Ident, Item, LitBool, LitInt, Token,
};

use crate::{
    ast::{App, AppArgs, HardwareTaskArgs, InitArgs, MonotonicArgs, SoftwareTaskArgs},
    Either, Settings,
};

// Parse the app, both app arguments and body (input)
pub fn app(args: TokenStream2, input: TokenStream2, settings: &Settings) -> parse::Result<App> {
    let args = AppArgs::parse(args)?;
    let input: Input = syn::parse2(input)?;

    App::parse(args, input, settings)
}

pub(crate) struct Input {
    _mod_token: Token![mod],
    pub ident: Ident,
    _brace_token: Brace,
    pub items: Vec<Item>,
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

        let _mod_token = input.parse()?;
        let ident = input.parse()?;
        let _brace_token = braced!(content in input);
        let items = content.call(parse_items)?;

        Ok(Input {
            _mod_token,
            ident,
            _brace_token,
            items,
        })
    }
}

fn init_idle_args(tokens: TokenStream2) -> parse::Result<InitArgs> {
    (|input: ParseStream<'_>| -> parse::Result<InitArgs> {
        if input.is_empty() {
            return Ok(InitArgs::default());
        }

        let mut late = None;
        let mut resources = None;

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
            late: late.unwrap_or_default(),

            resources: resources.unwrap_or_default(),
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
        let priority = priority.unwrap_or(1);
        let resources = resources.unwrap_or_default();

        Ok(if let Some(binds) = binds {
            Either::Left(HardwareTaskArgs {
                binds,
                priority,
                resources,
            })
        } else {
            Either::Right(SoftwareTaskArgs {
                capacity: capacity.unwrap_or(1),
                priority,
                resources,
            })
        })
    })
    .parse2(tokens)
}

fn monotonic_args(tokens: TokenStream2) -> parse::Result<MonotonicArgs> {
    (|input: ParseStream<'_>| -> parse::Result<MonotonicArgs> {
        let mut binds = None;
        let mut priority = None;
        let mut default = None;

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
                "binds" => {
                    if binds.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    // #ident
                    let ident = content.parse()?;

                    binds = Some(ident);
                }

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

                "default" => {
                    if default.is_some() {
                        return Err(parse::Error::new(
                            ident.span(),
                            "argument appears more than once",
                        ));
                    }

                    let lit: LitBool = content.parse()?;
                    default = Some(lit.value);
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
        let binds = if let Some(r) = binds {
            r
        } else {
            return Err(parse::Error::new(content.span(), "`binds = ...` is missing"));
        };
        let priority = priority.unwrap_or(1);
        let default = default.unwrap_or(false);

        Ok(MonotonicArgs {
            binds,
            priority,
            default,
        })
    })
    .parse2(tokens)
}
