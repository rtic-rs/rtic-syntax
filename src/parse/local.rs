use indexmap::map::Entry;
use syn::{parse, ItemStatic};

use crate::{ast::Local, parse::util, Map};

impl Local {
    pub(crate) fn parse(items: Vec<ItemStatic>, cores: u8) -> parse::Result<Map<Local>> {
        let mut locals = Map::new();

        for item in items {
            let span = item.ident.span();
            match locals.entry(item.ident) {
                Entry::Occupied(..) => {
                    return Err(parse::Error::new(
                        span,
                        "this `static` is listed more than once",
                    ));
                }

                Entry::Vacant(entry) => {
                    let (cfgs, mut attrs) = util::extract_cfgs(item.attrs);

                    let shared = util::extract_shared(&mut attrs, cores)?;

                    entry.insert(Local {
                        attrs,
                        cfgs,
                        shared,
                        expr: item.expr,
                        ty: item.ty,
                        _extensible: (),
                    });
                }
            }
        }

        Ok(locals)
    }
}
