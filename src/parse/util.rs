use syn::{
    bracketed,
    parse::{self, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Abi, AttrStyle, Attribute, Expr, FnArg, ForeignItemFn, Ident, ItemFn, Pat, PatType, Path,
    PathArguments, ReturnType, Token, Type, Visibility,
};

use crate::{
    ast::{Access, Local, LocalResources, SharedResources, TaskLocal},
    Map,
};

pub fn abi_is_rust(abi: &Abi) -> bool {
    match &abi.name {
        None => true,
        Some(s) => s.value() == "Rust",
    }
}

pub fn attr_eq(attr: &Attribute, name: &str) -> bool {
    attr.style == AttrStyle::Outer && attr.path.segments.len() == 1 && {
        let segment = attr.path.segments.first().unwrap();
        segment.arguments == PathArguments::None && *segment.ident.to_string() == *name
    }
}

/// checks that a function signature
///
/// - has no bounds (like where clauses)
/// - is not `async`
/// - is not `const`
/// - is not `unsafe`
/// - is not generic (has no type parameters)
/// - is not variadic
/// - uses the Rust ABI (and not e.g. "C")
pub fn check_fn_signature(item: &ItemFn) -> bool {
    item.vis == Visibility::Inherited
        && item.sig.constness.is_none()
        && item.sig.asyncness.is_none()
        && item.sig.abi.is_none()
        && item.sig.unsafety.is_none()
        && item.sig.generics.params.is_empty()
        && item.sig.generics.where_clause.is_none()
        && item.sig.variadic.is_none()
}

#[allow(dead_code)]
pub fn check_foreign_fn_signature(item: &ForeignItemFn) -> bool {
    item.vis == Visibility::Inherited
        && item.sig.constness.is_none()
        && item.sig.asyncness.is_none()
        && item.sig.abi.is_none()
        && item.sig.unsafety.is_none()
        && item.sig.generics.params.is_empty()
        && item.sig.generics.where_clause.is_none()
        && item.sig.variadic.is_none()
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

pub fn extract_lock_free(attrs: &mut Vec<Attribute>) -> parse::Result<bool> {
    if let Some(pos) = attrs.iter().position(|attr| attr_eq(attr, "lock_free")) {
        attrs.remove(pos);
        Ok(true)
    } else {
        Ok(false)
    }
}

// pub fn parse_idents(content: ParseStream<'_>) -> parse::Result<Set<Ident>> {
//     let inner;
//     bracketed!(inner in content);
//
//     let mut idents = Set::new();
//     for ident in inner.call(Punctuated::<Ident, Token![,]>::parse_terminated)? {
//         if idents.contains(&ident) {
//             return Err(parse::Error::new(
//                 ident.span(),
//                 "identifier appears more than once in list",
//             ));
//         }
//
//         idents.insert(ident);
//     }
//
//     Ok(idents)
// }

pub fn parse_shared_resources(content: ParseStream<'_>) -> parse::Result<SharedResources> {
    let inner;
    bracketed!(inner in content);

    let mut resources = Map::new();
    for e in inner.call(Punctuated::<Expr, Token![,]>::parse_terminated)? {
        let err = Err(parse::Error::new(
            e.span(),
            "identifier appears more than once in list",
        ));
        let (access, path) = match e {
            Expr::Path(e) => (Access::Exclusive, e.path),

            Expr::Reference(ref r) if r.mutability.is_none() => match &*r.expr {
                Expr::Path(e) => (Access::Shared, e.path.clone()),

                _ => return err,
            },

            _ => return err,
        };

        let ident = extract_resource_name_ident(path)?;

        if resources.contains_key(&ident) {
            return Err(parse::Error::new(
                ident.span(),
                "resource appears more than once in list",
            ));
        }

        resources.insert(ident, access);
    }

    Ok(resources)
}

fn extract_resource_name_ident(path: Path) -> parse::Result<Ident> {
    if path.leading_colon.is_some()
        || path.segments.len() != 1
        || path.segments[0].arguments != PathArguments::None
    {
        Err(parse::Error::new(
            path.span(),
            "resource must be an identifier, not a path",
        ))
    } else {
        Ok(path.segments[0].ident.clone())
    }
}

pub fn parse_local_resources(content: ParseStream<'_>) -> parse::Result<LocalResources> {
    let inner;
    bracketed!(inner in content);

    let mut resources = Map::new();

    for e in inner.call(Punctuated::<Expr, Token![,]>::parse_terminated)? {
        let err = Err(parse::Error::new(
            e.span(),
            "identifier appears more than once in list",
        ));

        // println!("e: {:#?}", e);

        let (name, local) = match e {
            // local = [IDENT],
            Expr::Path(path) => {
                let ident = extract_resource_name_ident(path.path)?;

                (ident, TaskLocal::External)
            }

            // local = [IDENT: TYPE = EXPR]
            Expr::Assign(e) => {
                let (name, ty) = match *e.left {
                    Expr::Type(t) => {
                        // Extract name
                        let name = match *t.expr {
                            Expr::Path(path) => extract_resource_name_ident(path.path)?,
                            _ => return err,
                        };

                        let ty = t.ty;

                        // Error check
                        match &*ty {
                            Type::Array(_) => {}
                            Type::Path(_) => {}
                            Type::Ptr(_) => {}
                            Type::Tuple(_) => {}
                            _ => return Err(parse::Error::new(
                                ty.span(),
                                "unsupported type, must be an array, tuple, pointer or type path",
                            )),
                        };

                        (name, ty)
                    }
                    _ => return Err(parse::Error::new(e.span(), "not a type")),
                };

                let expr = e.right; // Expr

                (
                    name,
                    TaskLocal::Declared(Local {
                        attrs: Vec::new(),
                        cfgs: Vec::new(),
                        ty,
                        expr,
                    }),
                )
            }

            _ => return err,
        };

        resources.insert(name, local);
    }

    Ok(resources)
}

type ParseInputResult = Option<(Box<Pat>, Result<Vec<PatType>, FnArg>)>;

pub fn parse_inputs(inputs: Punctuated<FnArg, Token![,]>, name: &str) -> ParseInputResult {
    let mut inputs = inputs.into_iter();

    match inputs.next() {
        Some(FnArg::Typed(first)) => {
            if type_is_path(&first.ty, &[name, "Context"]) {
                let rest = inputs
                    .map(|arg| match arg {
                        FnArg::Typed(arg) => Ok(arg),
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
        matches!(**ty, Type::Never(_))
    } else {
        false
    }
}

fn extract_init_resource_name_ident(ty: Type) -> Result<Ident, ()> {
    match ty {
        Type::Path(path) => {
            let path = path.path;

            if path.leading_colon.is_some()
                || path.segments.len() != 1
                || path.segments[0].arguments != PathArguments::None
            {
                Err(())
            } else {
                Ok(path.segments[0].ident.clone())
            }
        }
        _ => Err(()),
    }
}

/// Checks Init's return type and returns the user provided types for use in the analysis
pub fn type_is_init_return(ty: &ReturnType, name: &str) -> Result<(Ident, Ident), ()> {
    match ty {
        ReturnType::Default => Err(()),

        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(t) => {
                // return should be:
                // fn -> (User's #[shared] struct, User's #[local] struct, {name}::Monotonics)
                //
                // We check the length and the last one here, analysis checks that the user
                // provided structs are correct.
                if t.elems.len() == 3 {
                    if type_is_path(&t.elems[2], &[name, "Monotonics"]) {
                        return Ok((
                            extract_init_resource_name_ident(t.elems[0].clone())?,
                            extract_init_resource_name_ident(t.elems[1].clone())?,
                        ));
                    }
                }

                Err(())
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
