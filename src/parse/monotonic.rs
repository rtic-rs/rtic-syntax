use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse, ItemType, Visibility};

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

        let (cfgs, attrs) = util::extract_cfgs(item.attrs.clone());

        Ok(Monotonic {
            cfgs,
            attrs,
            ty: item.ty.clone(),
            args,
            // properties: ResourceProperties {
            //     task_local,
            //     lock_free,
            // },
        })
    }
}
