use proc_macro2::{Span, TokenStream as TokenStream2};

use syn::{parse, ItemFn};

use crate::{
    ast::{HardwareTask, HardwareTaskArgs, HardwareTaskKind, Local},
    parse::util,
    Settings,
};

impl HardwareTaskArgs {
    pub(crate) fn parse(
        cores: u8,
        tokens: TokenStream2,
        settings: &Settings,
        span: Span,
    ) -> parse::Result<Self> {
        crate::parse::task_args(
            tokens, cores, settings, /* accepts_binds */ true,
            /* accepts_capacity */ false,
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
                binds: args.binds,
                priority: args.priority,
                resources: args.resources,
                spawn: args.spawn,
                schedule: args.schedule,
                _extensible: (),
            })
        })
    }
}

impl HardwareTask {
    pub(crate) fn parse(
        args: HardwareTaskArgs,
        kind: HardwareTaskKind,
        item: ItemFn,
    ) -> parse::Result<Self> {
        let span = item.ident.span();
        let valid_signature = util::check_fn_signature(&item)
            && item.decl.inputs.len() == 1
            && util::type_is_unit(&item.decl.output);

        let name = item.ident.to_string();

        if name == "init" || name == "idle" {
            return Err(parse::Error::new(
                span,
                "tasks cannot be named `init` or `idle`",
            ));
        }

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.decl.inputs, &name) {
                if rest.is_empty() {
                    let (locals, stmts) = util::extract_locals(item.block.stmts)?;
                    let attrs = item.attrs;

                    return Ok(HardwareTask {
                        args,
                        attrs,
                        context,
                        kind,
                        locals: Local::parse(locals)?,
                        stmts,
                        _extensible: (),
                    });
                }
            }
        }

        Err(parse::Error::new(
            span,
            &format!(
                "this task handler must have type signature `fn({}::Context)`",
                name
            ),
        ))
    }
}
