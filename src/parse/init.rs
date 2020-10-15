use proc_macro2::TokenStream as TokenStream2;

use syn::{parse, ItemFn};

use crate::{
    ast::{Init, InitArgs, Local},
    parse::util,
};

impl InitArgs {
    pub(crate) fn parse(tokens: TokenStream2) -> parse::Result<Self> {
        crate::parse::init_idle_args(tokens)
    }
}

impl Init {
    pub(crate) fn parse(args: InitArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature = util::check_fn_signature(&item) && item.sig.inputs.len() == 1;

        let span = item.sig.ident.span();

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Ok(returns_late_resources) =
                util::type_is_late_resources(&item.sig.output, &name)
            {
                if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                    if rest.is_empty() && returns_late_resources {
                        let (locals, stmts) = util::extract_locals(item.block.stmts)?;

                        return Ok(Init {
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
        }

        Err(parse::Error::new(
            span,
            &format!(
                "this `#[init]` function must have signature `fn({}::Context) -> {0}::LateResources`",
                name
            ),
        ))
    }
}
