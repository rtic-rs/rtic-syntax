//! Parser of the `app!` macro used by the Real Time For the Masses (RTFM)
//! framework
#![feature(proc_macro)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(warnings)]

#[macro_use]
extern crate error_chain;
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

pub mod check;
mod parse;
mod util;
pub mod error;

use proc_macro::TokenStream;
use std::collections::{HashMap, HashSet};
use syn::{Ident, Path};
use syn::synom::ParseError;
use syn::ItemStatic;

/// `[$($ident),*]`
pub type Resources = HashSet<Ident>;

/// `$(static $Ident: $Ty = $expr;)*`
pub type Statics = HashMap<Ident, ItemStatic>;

/// `$($Ident: { .. },)*`
pub type Tasks = HashMap<Ident, Task>;

/// `app! { .. }`
#[derive(Debug, Default)]
pub struct App {
    /// `device: $path`
    pub device: Option<Path>,
    /// `idle: { $Idle }`
    pub idle: Option<Idle>,
    /// `init: { $Init }`
    pub init: Option<Init>,
    /// `resources: $Statics`
    pub resources: Option<Statics>,
    /// `tasks: { $Tasks }`
    pub tasks: Option<Tasks>,
    _extensible: (),
}

/// `idle: { .. }`
#[derive(Debug, Default)]
pub struct Idle {
    /// `path: $Path`
    pub path: Option<Path>,
    /// `resources: $Resources`
    pub resources: Option<Resources>,
    _extensible: (),
}

/// `init: { .. }`
#[derive(Debug, Default)]
pub struct Init {
    /// `path: $Path`
    pub path: Option<Path>,
    /// `resources: $Resources`
    pub resources: Option<Resources>,
    _extensible: (),
}

/// `$Ident: { .. }`
#[derive(Debug, Default)]
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

impl App {
    /// Parses the contents of the `app! { .. }` macro
    pub fn parse(input: TokenStream) -> Result<Self, ParseError> {
        syn::parse(input)
    }
}
