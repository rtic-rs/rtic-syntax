use syn::{Ident, Path, PathParameters, PathSegment};

/// Creates a path with contents `#ident`
pub fn mk_path(ident: &str) -> Path {
    Path {
        global: false,
        segments: vec![
            PathSegment {
                ident: Ident::new(ident),
                parameters: PathParameters::none(),
            },
        ],
    }
}
