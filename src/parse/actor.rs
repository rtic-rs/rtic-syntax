use proc_macro2::{Ident, Span};
use syn::{
    parenthesized,
    parse::{self, Parse},
    spanned::Spanned,
    Field, LitInt, Token, Type, Visibility,
};

use crate::ast::{Actor, Subscription};

use super::util::{self, FilterAttrs};

impl Actor {
    pub(crate) fn parse(item: &Field, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let FilterAttrs { cfgs, attrs, .. } = util::filter_attributes(item.attrs.clone());

        if !cfgs.is_empty() {
            return Err(parse::Error::new(span, "`#[cfg]` is not allowed on actors"));
        }

        let mut priority = None;
        let mut init = None;
        let mut subscriptions = Vec::new();

        for attr in attrs {
            match attr.path.get_ident() {
                Some(name) => {
                    match &*name.to_string() {
                        "priority" => {
                            if priority.is_some() {
                                return Err(parse::Error::new(
                                    attr.span(),
                                    "only one `#[priority]` attribute is allowed on an actor",
                                ));
                            }

                            let prio: EqPriority = syn::parse2(attr.tokens)?;
                            priority = Some(prio.priority);
                        }
                        "init" => {
                            if init.is_some() {
                                return Err(parse::Error::new(
                                    attr.span(),
                                    "only one `#[init]` attribute is allowed on an actor",
                                ));
                            }

                            // `#[init(expr)]` can be parsed via `ExprParen`
                            let paren: syn::ExprParen = syn::parse2(attr.tokens)?;

                            init = Some(paren.expr);
                        }
                        "subscribe" => {
                            let subscribe: Subscribe = syn::parse2(attr.tokens)?;
                            let capacity = subscribe
                                .capacity
                                .map(|lit| {
                                    lit.base10_digits().parse::<u8>().map_err(|_| {
                                        parse::Error::new(lit.span(), "not a `u8` value")
                                    })
                                })
                                .transpose()?;

                            subscriptions.push(Subscription {
                                ty: subscribe.ty,
                                capacity: capacity.unwrap_or(1),
                            });
                        }
                        _ => {
                            return Err(parse::Error::new(
                                name.span(),
                                "this attribute is not supported on actor declarations",
                            ));
                        }
                    }
                }
                None => {
                    return Err(parse::Error::new(
                        attr.path.span(),
                        "this attribute is not supported on actor declarations",
                    ));
                }
            }
        }

        Ok(Actor {
            ty: Box::new(item.ty.clone()),
            priority: priority.unwrap_or(1),
            init,
            subscriptions,
        })
    }
}

struct EqPriority {
    priority: u8,
}

impl parse::Parse for EqPriority {
    fn parse(input: parse::ParseStream<'_>) -> syn::Result<Self> {
        let _eq: Token![=] = input.parse()?;
        let lit: syn::LitInt = input.parse()?;

        if !lit.suffix().is_empty() {
            return Err(parse::Error::new(
                lit.span(),
                "this literal must be unsuffixed",
            ));
        }

        let value = lit.base10_parse::<u8>().ok();
        match value {
            None | Some(0) => Err(parse::Error::new(
                lit.span(),
                "this literal must be in the range 1...255",
            )),
            Some(priority) => Ok(Self { priority }),
        }
    }
}

struct Subscribe {
    ty: Type,
    capacity: Option<LitInt>,
}

impl Parse for Subscribe {
    fn parse(input: parse::ParseStream<'_>) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let ty = content.parse()?;

        let capacity = if content.is_empty() {
            None
        } else {
            let _: Token![,] = content.parse()?;
            let ident: Ident = content.parse()?;

            if ident.to_string() == "capacity" {
                let _: Token![=] = content.parse()?;
                Some(content.parse()?)
            } else {
                return Err(parse::Error::new(
                    ident.span(),
                    format!("expected `capacity`, found `{}`", ident),
                ));
            }
        };

        Ok(Self { ty, capacity })
    }
}
