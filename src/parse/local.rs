use indexmap::map::Entry;
use syn::{parse, ItemStatic};

use crate::{ast::Local, parse::util, Map};

impl Local {
    pub(crate) fn parse(items: Vec<ItemStatic>) -> parse::Result<Map<Local>> {
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
                    let (cfgs, attrs) = util::extract_cfgs(item.attrs);

                    entry.insert(Local {
                        attrs,
                        cfgs,
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
