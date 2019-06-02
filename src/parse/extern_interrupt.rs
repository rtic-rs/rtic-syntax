use syn::{parse, ForeignItemFn};

use crate::{ast::ExternInterrupt, parse::util};

impl ExternInterrupt {
    pub(crate) fn parse(item: &ForeignItemFn) -> parse::Result<ExternInterrupt> {
        let valid_signature = util::check_foreign_fn_signature(&item)
            && item.decl.inputs.is_empty()
            && util::type_is_unit(&item.decl.output);

        if !valid_signature {
            return Err(parse::Error::new(
                item.ident.span(),
                "extern interrupts must have type signature `fn()`",
            ));
        }

        Ok(ExternInterrupt {
            attrs: item.attrs.clone(),
            _extensible: (),
        })
    }
}
