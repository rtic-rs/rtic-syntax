use proc_macro2::Span;
use syn::{parse, Field, Visibility};

use crate::{ast::LateResource, parse::util};

impl LateResource {
    pub(crate) fn parse(item: &Field, span: Span, cores: u8) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let (cfgs, mut attrs) = util::extract_cfgs(item.attrs.clone());

        let shared = util::extract_shared(&mut attrs, cores)?;

        Ok(LateResource {
            cfgs,
            attrs,
            shared,
            ty: Box::new(item.ty.clone()),
            _extensible: (),
        })
    }
}
