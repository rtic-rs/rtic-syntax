use syn::{parse, spanned::Spanned, ItemStatic, Visibility};

use crate::{
    ast::{LateResource, Resource},
    parse::util,
};

impl Resource {
    pub(crate) fn parse(item: ItemStatic) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                item.span(),
                "resources must have inherited / private visibility",
            ));
        }

        let (cfgs, attrs) = util::extract_cfgs(item.attrs);

        Ok(Resource {
            late: LateResource {
                cfgs,
                attrs,
                mutability: item.mutability,
                ty: item.ty,
                _extensible: (),
            },
            expr: item.expr,
        })
    }
}
