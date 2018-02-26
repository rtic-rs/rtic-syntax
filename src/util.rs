use syn::{Ident, Path, PathSegment, PathArguments};
use syn::punctuated::Punctuated;

/// Creates a path with contents `#ident`
pub fn mk_path(ident: &str) -> Path {
    let mut segment = Punctuated::new();
    segment.push(PathSegment {
                ident: Ident::from(ident),
                arguments: PathArguments::None,
    });
    Path {
        leading_colon: None,
        segments: segment
    }
}
