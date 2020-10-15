use proc_macro2::Span;
use syn::{parse, Field, Visibility};

use crate::{
    ast::{LateResource, ResourceProperties},
    parse::util,
};

impl LateResource {
    pub(crate) fn parse(item: &Field, span: Span) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                span,
                "this field must have inherited / private visibility",
            ));
        }

        let (cfgs, mut attrs) = util::extract_cfgs(item.attrs.clone());

        let task_local = util::extract_task_local(&mut attrs)?;

        let lock_free = util::extract_lock_free(&mut attrs)?;

        Ok(LateResource {
            cfgs,
            attrs,
            ty: Box::new(item.ty.clone()),
            properties: ResourceProperties {
                task_local,
                lock_free,
            },
            _extensible: (),
        })
    }
}
