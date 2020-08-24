use syn::{parse, ForeignItemFn, Ident};

use crate::{ast::ExternInterrupt, parse::util, Id};

impl ExternInterrupt {
    pub(crate) fn parse(
        item: ForeignItemFn,
    ) -> parse::Result<(Id, Ident, ExternInterrupt)> {
        let valid_signature = util::check_foreign_fn_signature(&item)
            && item.sig.inputs.is_empty()
            && util::type_is_unit(&item.sig.output);

        if !valid_signature {
            return Err(parse::Error::new(
                item.sig.ident.span(),
                "extern interrupts must have type signature `fn()`",
            ));
        }

        let attrs = item.attrs;

        Ok((
            0,
            item.sig.ident,
            ExternInterrupt {
                attrs,
                _extensible: (),
            },
        ))
    }
}
