use proc_macro2::Span;
use syn::{parse, Field, Visibility};

use crate::{
    ast::{LocalResource, SharedResource, SharedResourceProperties},
    parse::util,
};

impl SharedResource {
    pub(crate) fn parse(item: &Field, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let (cfgs, mut attrs) = util::extract_cfgs(item.attrs.clone());

        let lock_free = util::extract_lock_free(&mut attrs)?;

        Ok(SharedResource {
            cfgs,
            attrs,
            ty: Box::new(item.ty.clone()),
            properties: SharedResourceProperties { lock_free },
        })
    }
}

impl LocalResource {
    pub(crate) fn parse(item: &Field, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let (cfgs, attrs) = util::extract_cfgs(item.attrs.clone());

        Ok(LocalResource {
            cfgs,
            attrs,
            ty: Box::new(item.ty.clone()),
        })
    }
}
