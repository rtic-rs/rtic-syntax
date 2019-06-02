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
    pub(crate) fn parse(args: IdleArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item)
            && item.decl.inputs.len() == 1
            && util::type_is_bottom(&item.decl.output);

        let name = item.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.decl.inputs, &name) {
                if rest.is_empty() {
                    let (locals, stmts) = util::extract_locals(item.block.stmts)?;

                    return Ok(Idle {
                        args,
                        attrs: item.attrs,
                        context,
                        locals: Local::parse(locals)?,
                        name: item.ident,
                        stmts,
                        _extensible: (),
                    });
                }
            }
        }

        Err(parse::Error::new(
            item.ident.span(),
            &format!(
                "this `#[idle]` must have type signature `fn({}::Context) -> !`",
                name
            ),
        ))
    }
}
