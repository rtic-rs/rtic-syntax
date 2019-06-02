use syn::{parse, spanned::Spanned, ForeignItemStatic, Visibility};

use crate::{ast::LateResource, parse::util};

impl LateResource {
    pub(crate) fn parse(item: ForeignItemStatic) -> parse::Result<Self> {
        if item.vis != Visibility::Inherited {
            return Err(parse::Error::new(
                item.span(),
                "resources must have inherited / private visibility",
            ));
        }

        let (cfgs, attrs) = util::extract_cfgs(item.attrs);

        Ok(LateResource {
            cfgs,
            attrs,
            mutability: item.mutability,
            ty: item.ty,
            _extensible: (),
        })
    }
}
