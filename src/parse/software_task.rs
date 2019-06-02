use proc_macro2::{Span, TokenStream as TokenStream2};

use syn::{parse, ItemFn};

use crate::{
    ast::{Local, SoftwareTask, SoftwareTaskArgs},
    parse::util,
    Settings,
};

impl SoftwareTaskArgs {
    pub(crate) fn parse(
        cores: u8,
        tokens: TokenStream2,
        settings: &Settings,
        span: Span,
    ) -> parse::Result<Self> {
        crate::parse::task_args(
            tokens, cores, settings, /* accepts_binds */ false,
            /* accepts_capacity */ true,
        )
        .and_then(|args| {
            Ok(Self {
                core: if cores == 1 {
                    0
                } else {
                    if let Some(core) = args.core {
                        core
                    } else {
                        return Err(parse::Error::new(
                            span,
                            "this task must be assigned to a core",
                        ));
                    }
                },
                capacity: args.capacity.unwrap_or(1),
                priority: args.priority,
                resources: args.resources,
                spawn: args.spawn,
                schedule: args.schedule,
                _extensible: (),
            })
        })
    }
}

impl SoftwareTask {
    pub(crate) fn parse(args: SoftwareTaskArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature =
            util::check_fn_signature(&item) && util::type_is_unit(&item.decl.output);

        let span = item.ident.span();

        let name = item.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(inputs))) = util::parse_inputs(item.decl.inputs, &name) {
                let (locals, stmts) = util::extract_locals(item.block.stmts)?;
                let (cfgs, attrs) = util::extract_cfgs(item.attrs);

                return Ok(SoftwareTask {
                    args,
                    attrs,
                    cfgs,
                    context,
                    inputs,
                    locals: Local::parse(locals)?,
                    stmts,
                    _extensible: (),
                });
            }
        }

        Err(parse::Error::new(
            span,
            &format!(
                "this task handler must have type signature `fn({}::Context, ..)`",
                name
            ),
        ))
    }
}
