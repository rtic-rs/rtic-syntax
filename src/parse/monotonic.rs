use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse, spanned::Spanned, ItemType, Visibility};

use crate::parse::util::FilterAttrs;
use crate::{
    ast::{Monotonic, MonotonicArgs},
    parse::util,
};

impl MonotonicArgs {
    pub(crate) fn parse(tokens: TokenStream2) -> parse::Result<Self> {
        crate::parse::monotonic_args(tokens)
    }
}

impl Monotonic {
    pub(crate) fn parse(args: MonotonicArgs, item: &ItemType, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let FilterAttrs { cfgs, attrs, .. } = util::filter_attributes(item.attrs.clone());

        if attrs.len() > 0 {
            return Err(parse::Error::new(
                attrs[0].path.span(),
                "Monotonic does not support attributes other than `#[cfg]`",
            ));
        }

        Ok(Monotonic {
            cfgs,
            ident: item.ident.clone(),
            ty: item.ty.clone(),
            args,
        })
    }
}
