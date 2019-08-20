use syn::{parse, ForeignItemFn, Ident};

use crate::{ast::ExternInterrupt, parse::util, Core};

impl ExternInterrupt {
    pub(crate) fn parse(
        item: ForeignItemFn,
        cores: u8,
    ) -> parse::Result<(Core, Ident, ExternInterrupt)> {
        let valid_signature = util::check_foreign_fn_signature(&item)
            && item.sig.inputs.is_empty()
            && util::type_is_unit(&item.sig.output);

        if !valid_signature {
            return Err(parse::Error::new(
                item.sig.ident.span(),
                "extern interrupts must have type signature `fn()`",
            ));
        }

        let (core, attrs) = if cores > 1 {
            util::extract_core(item.attrs, cores, item.sig.ident.span())?
        } else {
            (0, item.attrs)
        };

        Ok((
            core,
            item.sig.ident,
            ExternInterrupt {
                attrs,
                _extensible: (),
            },
        ))
    }
}
