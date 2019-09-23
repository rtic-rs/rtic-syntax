use syn::{parse, ItemFn};

use crate::{
    ast::{HardwareTask, HardwareTaskArgs, Local},
    parse::util,
    Settings,
};

impl HardwareTask {
    pub(crate) fn parse(
        args: HardwareTaskArgs,
        item: ItemFn,
        cores: u8,
        settings: &Settings,
    ) -> parse::Result<Self> {
        let span = item.sig.ident.span();
        let valid_signature = util::check_fn_signature(&item) && item.sig.inputs.len() == 1;

        let name = item.sig.ident.to_string();

        if name == "init" || name == "idle" {
            return Err(parse::Error::new(
                span,
                "tasks cannot be named `init` or `idle`",
            ));
        }

        if valid_signature {
            if let Some((context, Ok(rest))) = util::parse_inputs(item.sig.inputs, &name) {
                if rest.is_empty() {
                    let (locals, stmts) = util::extract_locals(item.block.stmts)?;
                    let attrs = item.attrs;

                    let is_generator = settings.parse_impl_generator
                        && util::type_is_impl_generator(&item.sig.output);
                    if util::return_type_is_unit(&item.sig.output) || is_generator {
                        return Ok(HardwareTask {
                            args,
                            attrs,
                            context,
                            locals: Local::parse(locals, cores)?,
                            stmts,
                            is_generator,
                            _extensible: (),
                        });
                    }
                }
            }
        }

        let return_type = if settings.parse_impl_generator {
            " [-> impl Generator<Yield = (), Return = !>]"
        } else {
            ""
        };
        Err(parse::Error::new(
            span,
            &format!(
                "this task handler must have type signature `fn({}::Context){}`",
                name, return_type
            ),
        ))
    }
}
