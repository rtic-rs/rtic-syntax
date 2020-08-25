use proc_macro2::TokenStream as TokenStream2;
use syn::{parse, ItemFn};

use crate::{
    ast::{Idle, IdleArgs, Local},
    parse::util,
    Settings,
};

impl IdleArgs {
    pub(crate) fn parse(
        tokens: TokenStream2,
        settings: &Settings,
    ) -> parse::Result<Self> {
        crate::parse::init_idle_args(tokens, settings).map(|args| IdleArgs {
            resources: args.resources,
            spawn: args.spawn,
            schedule: args.schedule,
            _extensible: (),
        })
    }
}

impl Idle {
    pub(crate) fn parse(args: IdleArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item)
            && item.sig.inputs.len() == 1
            && util::type_is_bottom(&item.sig.output);

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    let (locals, stmts) = util::extract_locals(item.block.stmts)?;

                    return Ok(Idle {
                        args,
                        attrs: item.attrs,
                        context,
                        locals: Local::parse(locals)?,
                        name: item.sig.ident,
                        stmts,
                        _extensible: (),
                    });
                }
            }
        }

        Err(parse::Error::new(
            item.sig.ident.span(),
            &format!(
                "this `#[idle]` function must have signature `fn({}::Context) -> !`",
                name
            ),
        ))
    }
}
