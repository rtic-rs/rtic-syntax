#![deny(warnings)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate quote;
extern crate syn;

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
    pub idle: Idle,
    pub init: Init,
    pub resources: Statics,
    pub tasks: Tasks,
}

/// `init`
#[derive(Debug)]
pub struct Init {
    pub path: Tokens,
}

/// `idle`
#[derive(Debug)]
pub struct Idle {
    pub locals: Statics,
    pub path: Tokens,
    pub resources: Idents,
}

#[derive(Debug)]
pub struct Task {
    pub enabled: Option<bool>,
    pub priority: Option<u8>,
    pub resources: Idents,
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
