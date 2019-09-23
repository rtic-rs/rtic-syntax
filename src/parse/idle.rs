use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{parse, ItemFn};

use crate::{
    ast::{Idle, IdleArgs, Local},
    parse::util,
    Settings,
};

impl IdleArgs {
    pub(crate) fn parse(
        cores: u8,
        tokens: TokenStream2,
        settings: &Settings,
        span: Span,
    ) -> parse::Result<Self> {
        crate::parse::init_idle_args(tokens, cores, settings, false, span).map(|args| IdleArgs {
            core: args.core,
            resources: args.resources,
            spawn: args.spawn,
            schedule: args.schedule,
            _extensible: (),
        })
    }
}

impl Idle {
    pub(crate) fn parse(args: IdleArgs, item: ItemFn, cores: u8) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item)
            && item.sig.inputs.len() == 1
            && util::return_type_is_bottom(&item.sig.output);

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    let (locals, stmts) = util::extract_locals(item.block.stmts)?;

                    return Ok(Idle {
                        args,
                        attrs: item.attrs,
                        context,
                        locals: Local::parse(locals, cores)?,
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
