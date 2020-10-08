//! The Real-Time Interrupt-driven Concurrency (RTIC) meta language

#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

#[allow(unused_extern_crates)]
extern crate proc_macro;

use core::ops;
use proc_macro::TokenStream;

use indexmap::{IndexMap, IndexSet};
use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;

use crate::ast::App;

mod accessors;
pub mod analyze;
pub mod ast;
mod check;
mod optimize;
mod parse;
#[cfg(test)]
mod tests;

/// An ordered map keyed by identifier
pub type Map<T> = IndexMap<Ident, T>;

/// An order set
pub type Set<T> = IndexSet<T>;

/// Immutable pointer
pub struct P<T> {
    ptr: Box<T>,
}

impl<T> P<T> {
    /// Boxes `x` making the value immutable
    pub fn new(x: T) -> P<T> {
        P { ptr: Box::new(x) }
    }
}

impl<T> ops::Deref for P<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.ptr
    }
}

/// Execution context
#[derive(Clone, Copy)]
pub enum Context<'a> {
    /// A hardware task: `#[exception]` or `#[interrupt]`
    HardwareTask(&'a Ident),

    /// The `idle` context
    Idle,

    /// The `init`-ialization function
    Init,

    /// A software task: `#[task]`
    SoftwareTask(&'a Ident),
}

impl<'a> Context<'a> {
    /// The identifier of this context
    pub fn ident(&self, app: &'a App) -> &'a Ident {
        match self {
            Context::HardwareTask(ident) => ident,
            Context::Idle => &app.idles.first().unwrap().name,
            Context::Init => &app.inits.first().unwrap().name,
            Context::SoftwareTask(ident) => ident,
        }
    }

    /// Is this the `idle` context?
    pub fn is_idle(&self) -> bool {
        if let Context::Idle = self {
            true
        } else {
            false
        }
    }

    /// Is this the `init`-ialization context?
    pub fn is_init(&self) -> bool {
        if let Context::Init = self {
            true
        } else {
            false
        }
    }

    /// Whether this context runs only once
    pub fn runs_once(&self) -> bool {
        self.is_init() || self.is_idle()
    }

    /// Whether this context has local `static` variables
    pub fn has_locals(&self, app: &App) -> bool {
        match *self {
            Context::HardwareTask(name) => !app.hardware_tasks[name].locals.is_empty(),
            Context::Idle => !app.idles.first().unwrap().locals.is_empty(),
            Context::Init => !app.inits.first().unwrap().locals.is_empty(),
            Context::SoftwareTask(name) => !app.software_tasks[name].locals.is_empty(),
        }
    }

    /// Whether this context has resources
    pub fn has_resources(&self, app: &App) -> bool {
        match *self {
            Context::HardwareTask(name) => !app.hardware_tasks[name].args.resources.is_empty(),
            Context::Idle => !app.idles.first().unwrap().args.resources.is_empty(),
            Context::Init => !app.inits.first().unwrap().args.resources.is_empty(),
            Context::SoftwareTask(name) => !app.software_tasks[name].args.resources.is_empty(),
        }
    }
}

/// Parser and optimizer configuration
#[derive(Default)]
pub struct Settings {
    /// Whether to accept the `binds` argument in `#[task]` or not
    pub parse_binds: bool,
    /// Whether to parse `extern` interrupts (functions) or not
    pub parse_extern_interrupt: bool,
    /// Whether to "compress" priorities or not
    pub optimize_priorities: bool,

    _extensible: (),
}

/// Parses the input of the `#[app]` attribute
pub fn parse(
    args: TokenStream,
    input: TokenStream,
    settings: Settings,
) -> Result<(P<ast::App>, P<analyze::Analysis>), syn::parse::Error> {
    parse2(args.into(), input.into(), settings)
}

/// `proc_macro2::TokenStream` version of `parse`
pub fn parse2(
    args: TokenStream2,
    input: TokenStream2,
    ref settings: Settings,
) -> Result<(P<ast::App>, P<analyze::Analysis>), syn::parse::Error> {
    let mut app = parse::app(args, input, settings)?;
    check::app(&app)?;
    optimize::app(&mut app, settings);
    let analysis = analyze::app(&app);

    Ok((P::new(app), P::new(analysis)))
}

enum Either<A, B> {
    Left(A),
    Right(B),
}
