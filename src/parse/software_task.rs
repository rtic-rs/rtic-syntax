use syn::{parse, ForeignItemFn, ItemFn, Stmt};

use crate::parse::util::FilterAttrs;
use crate::{
    ast::{SoftwareTask, SoftwareTaskArgs},
    parse::util,
};

impl SoftwareTask {
    pub(crate) fn parse(args: SoftwareTaskArgs, item: ItemFn) -> parse::Result<Self> {
        let valid_signature =
            util::check_fn_signature(&item) && util::type_is_unit(&item.sig.output);

        let span = item.sig.ident.span();

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(inputs))) = util::parse_inputs(item.sig.inputs, &name) {
                let FilterAttrs { cfgs, attrs, .. } = util::filter_attributes(item.attrs);

                return Ok(SoftwareTask {
                    args,
                    attrs,
                    cfgs,
                    context,
                    inputs,
                    stmts: item.block.stmts,
                    is_extern: false,
                });
            }
        }

        Err(parse::Error::new(
            span,
            format!(
                "this task handler must have type signature `fn({}::Context, ..)`",
                name
            ),
        ))
    }
}

impl SoftwareTask {
    pub(crate) fn parse_foreign(
        args: SoftwareTaskArgs,
        item: ForeignItemFn,
    ) -> parse::Result<Self> {
        let valid_signature =
            util::check_foreign_fn_signature(&item) && util::type_is_unit(&item.sig.output);

        let span = item.sig.ident.span();

        let name = item.sig.ident.to_string();

        if valid_signature {
            if let Some((context, Ok(inputs))) = util::parse_inputs(item.sig.inputs, &name) {
                let FilterAttrs { cfgs, attrs, .. } = util::filter_attributes(item.attrs);

                return Ok(SoftwareTask {
                    args,
                    attrs,
                    cfgs,
                    context,
                    inputs,
                    stmts: Vec::<Stmt>::new(),
                    is_extern: true,
                });
            }
        }

        Err(parse::Error::new(
            span,
            format!(
                "this task handler must have type signature `fn({}::Context, ..)`",
                name
            ),
        ))
    }
}
