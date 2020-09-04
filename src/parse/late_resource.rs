use proc_macro2::Span;
use syn::{parse, Field, Visibility};

use crate::{ast::LateResource, parse::util};

impl LateResource {
    pub(crate) fn parse(item: &Field, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let (cfgs, attrs) = util::extract_cfgs(item.attrs.clone());

        Ok(LateResource {
            cfgs,
            attrs,
            ty: Box::new(item.ty.clone()),
            _extensible: (),
        })
    }
}
