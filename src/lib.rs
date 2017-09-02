//! Parser of the `app!` macro used by the Real Time For the Masses (RTFM)
//! framework
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate quote;
extern crate syn;

pub mod check;
pub mod error;

mod parse;
mod util;

use std::collections::{HashMap, HashSet};

use quote::Tokens;
use syn::{Ident, Path, Ty};

use error::*;

/// A rust expression
pub type Expr = Tokens;

/// `[$($ident),*]`
pub type Resources = HashSet<Ident>;

/// `$(static $Ident: $Ty = $expr;)*`
pub type Statics = HashMap<Ident, Static>;

/// `$($Ident: { .. },)*`
pub type Tasks = HashMap<Ident, Task>;

/// `app! { .. }`
#[derive(Debug)]
pub struct App {
    /// `device: $path`
    pub device: Path,
    /// `idle: { $Idle }`
    pub idle: Option<Idle>,
    /// `init: { $Init }`
    pub init: Option<Init>,
    /// `resources: $Resources`
    pub resources: Option<Statics>,
    /// `tasks: { $Tasks }`
    pub tasks: Option<Tasks>,
    _extensible: (),
}

/// `idle: { .. }`
#[derive(Debug)]
pub struct Idle {
    /// `path: $Path`
    pub path: Option<Path>,
    /// `resources: $Resources`
    pub resources: Option<Resources>,
    _extensible: (),
}

/// `init: { .. }`
#[derive(Debug)]
pub struct Init {
    /// `path: $Path`
    pub path: Option<Path>,
    _extensible: (),
}

/// `$Ident: { .. }`
#[derive(Debug)]
pub struct Task {
    /// `enabled: $bool`
    pub enabled: Option<bool>,
    /// `path: $Path`
    pub path: Option<Path>,
    /// `priority: $u8`
    pub priority: Option<u8>,
    /// `resources: $Resources`
    pub resources: Option<Resources>,
    _extensible: (),
}

/// `static $Ident: $Ty = $Expr;`
#[derive(Debug)]
pub struct Static {
    /// `$Expr`
    pub expr: Option<Expr>,
    /// `$Ty`
    pub ty: Ty,
    _extensible: (),
}

impl App {
    /// Parses the contents of the `app! { .. }` macro
    pub fn parse(input: &str) -> Result<Self> {
        parse::app(input)
    }
}
