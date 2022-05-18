use proc_macro2::TokenStream as TokenStream2;
use syn::{parse, ItemFn};

use crate::{
    ast::{Idle, IdleArgs},
    parse::util,
};

impl IdleArgs {
    pub(crate) fn parse(tokens: TokenStream2) -> parse::Result<Self> {
        crate::parse::idle_args(tokens)
    }
}

impl Idle {
    pub(crate) fn parse(args: IdleArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item, true)
            && item.sig.inputs.len() == 1
            && util::type_is_bottom(&item.sig.output);

        let name = item.sig.ident.to_string();
        let is_async = item.sig.asyncness.is_some();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    return Ok(Idle {
                        args,
                        attrs: item.attrs,
                        context,
                        name: item.sig.ident,
                        stmts: item.block.stmts,
                        is_async,
                    });
                }
            }
        }

        Err(parse::Error::new(
            item.sig.ident.span(),
            &format!(
                "this `#[idle]` function must have signature `(async) fn({}::Context) -> !`",
                name
            ),
        ))
    }
}
