#![deny(warnings)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate quote;
extern crate syn;

pub mod check;
pub mod error;

mod parse;

use std::collections::{HashMap, HashSet};

use quote::Tokens;
use syn::Ident;

use error::*;

pub type Idents = HashSet<Ident>;

pub type Statics = HashMap<Ident, Static>;

pub type Tasks = HashMap<Ident, Task>;

#[derive(Debug)]
pub struct App {
    pub device: Tokens,
    pub idle: Option<Idle>,
    pub init: Option<Init>,
    pub resources: Option<Statics>,
    pub tasks: Option<Tasks>,
}

/// `init`
#[derive(Debug)]
pub struct Init {
    pub path: Option<Tokens>,
}

/// `idle`
#[derive(Debug)]
pub struct Idle {
    pub locals: Option<Statics>,
    pub path: Option<Tokens>,
    pub resources: Option<Idents>,
}

#[derive(Debug)]
pub struct Task {
    pub enabled: Option<bool>,
    pub priority: Option<u8>,
    pub resources: Option<Idents>,
}

// `$ident: $ty = $expr;`
#[derive(Debug)]
pub struct Static {
    pub expr: Tokens,
    pub ty: Tokens,
}

impl App {
    pub fn parse(input: &str) -> Result<Self> {
        parse::app(input)
    }
}
