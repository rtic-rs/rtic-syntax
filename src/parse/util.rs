use std::collections::HashSet;

use syn::{
    bracketed,
    parse::{self, ParseStream},
    punctuated::Punctuated,
    Abi, ArgCaptured, AttrStyle, Attribute, FnArg, ForeignItemFn, Ident, IntSuffix, Item, ItemFn,
    ItemStatic, LitInt, Pat, PathArguments, ReturnType, Stmt, Token, Type, Visibility,
};

use crate::Set;

pub fn abi_is_c(abi: &Abi) -> bool {
    match &abi.name {
        None => true,
        Some(s) => s.value() == "C",
    }
}

pub fn attr_eq(attr: &Attribute, name: &str) -> bool {
    attr.style == AttrStyle::Outer && attr.path.segments.len() == 1 && {
        let pair = attr.path.segments.first().unwrap();
        let segment = pair.value();
        segment.arguments == PathArguments::None && segment.ident.to_string() == name
    }
}

/// checks that a function signature
///
/// - has no bounds (like where clauses)
/// - is not `async`
/// - is not `const`
/// - is not `unsafe`
/// - is not generic (has no type parametrs)
/// - is not variadic
/// - uses the Rust ABI (and not e.g. "C")
pub fn check_fn_signature(item: &ItemFn) -> bool {
    item.vis == Visibility::Inherited
        && item.constness.is_none()
        && item.asyncness.is_none()
        && item.abi.is_none()
        && item.unsafety.is_none()
        && item.decl.generics.params.is_empty()
        && item.decl.generics.where_clause.is_none()
        && item.decl.variadic.is_none()
}

pub fn check_foreign_fn_signature(item: &ForeignItemFn) -> bool {
    item.vis == Visibility::Inherited
        // && item.constness.is_none()
        // && item.asyncness.is_none()
        // && item.abi.is_none()
        // && item.unsafety.is_none()
        && item.decl.generics.params.is_empty()
        && item.decl.generics.where_clause.is_none()
        && item.decl.variadic.is_none()
}

pub fn extract_cfgs(attrs: Vec<Attribute>) -> (Vec<Attribute>, Vec<Attribute>) {
    let mut cfgs = vec![];
    let mut not_cfgs = vec![];

    for attr in attrs {
        if attr_eq(&attr, "cfg") {
            cfgs.push(attr);
        } else {
            not_cfgs.push(attr);
        }
    }

    (cfgs, not_cfgs)
}

pub fn extract_locals(stmts: Vec<Stmt>) -> parse::Result<(Vec<ItemStatic>, Vec<Stmt>)> {
    let mut istmts = stmts.into_iter();

    let mut seen = HashSet::new();
    let mut locals = vec![];
    let mut stmts = vec![];
    while let Some(stmt) = istmts.next() {
        match stmt {
            Stmt::Item(Item::Static(static_)) => {
                if static_.mutability.is_some() {
                    if seen.contains(&static_.ident) {
                        return Err(parse::Error::new(
                            static_.ident.span(),
                            "this local `static` appears more than once",
                        ));
                    }

                    seen.insert(static_.ident.clone());
                    locals.push(static_);
                } else {
                    stmts.push(Stmt::Item(Item::Static(static_)));
                    break;
                }
            }

            _ => {
                stmts.push(stmt);
                break;
            }
        }
    }

    stmts.extend(istmts);

    Ok((locals, stmts))
}

pub fn parse_core(lit: LitInt, cores: u8) -> parse::Result<u8> {
    if lit.suffix() != IntSuffix::None {
        return Err(parse::Error::new(
            lit.span(),
            "this integer must be unsuffixed",
        ));
    }

    let val = lit.value();
    if val >= u64::from(cores) {
        return Err(parse::Error::new(
            lit.span(),
            &format!("core number must be in the range 0..{}", cores),
        ));
    }

    Ok(val as u8)
}

pub fn parse_idents(content: ParseStream<'_>) -> parse::Result<Set<Ident>> {
    let inner;
    bracketed!(inner in content);

    let mut idents = Set::new();
    for ident in inner.call(Punctuated::<Ident, Token![,]>::parse_terminated)? {
        if idents.contains(&ident) {
            return Err(parse::Error::new(
                ident.span(),
                "element appears more than once in list",
            ));
        }

        idents.insert(ident);
    }

    Ok(idents)
}

pub fn parse_inputs(
    inputs: Punctuated<FnArg, Token![,]>,
    name: &str,
) -> Option<(Pat, Result<Vec<ArgCaptured>, FnArg>)> {
    let mut inputs = inputs.into_iter();

    match inputs.next() {
        Some(FnArg::Captured(first)) => {
            if type_is_path(&first.ty, &[name, "Context"]) {
                let rest = inputs
                    .map(|arg| match arg {
                        FnArg::Captured(arg) => Ok(arg),
                        _ => Err(arg),
                    })
                    .collect::<Result<Vec<_>, _>>();

                Some((first.pat, rest))
            } else {
                None
            }
        }

        _ => None,
    }
}

pub fn type_is_bottom(ty: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ty {
        if let Type::Never(_) = **ty {
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn type_is_late_resources(ty: &ReturnType) -> Result<bool, ()> {
    match ty {
        ReturnType::Default => Ok(false),

        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(t) => {
                if t.elems.is_empty() {
                    Ok(false)
                } else {
                    Err(())
                }
            }

            Type::Path(_) => {
                if type_is_path(ty, &["init", "LateResources"]) {
                    Ok(true)
                } else {
                    Err(())
                }
            }

            _ => Err(()),
        },
    }
}

pub fn type_is_path(ty: &Type, segments: &[&str]) -> bool {
    match ty {
        Type::Path(tpath) if tpath.qself.is_none() => {
            tpath.path.segments.len() == segments.len()
                && tpath
                    .path
                    .segments
                    .iter()
                    .zip(segments)
                    .all(|(lhs, rhs)| lhs.ident == **rhs)
        }

        _ => false,
    }
}

pub fn type_is_unit(ty: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ty {
        if let Type::Tuple(ref tuple) = **ty {
            tuple.elems.is_empty()
        } else {
            false
        }
    } else {
        true
    }
}
