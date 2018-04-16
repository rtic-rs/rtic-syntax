//! Parser of the `app!` macro used by the Real Time For the Masses (RTFM) framework

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(proc_macro)]

extern crate either;
#[macro_use]
extern crate failure;
extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
#[macro_use]
extern crate syn;

use std::convert::TryInto;
use std::ops;

pub use failure::Error;

use either::Either;
use proc_macro2::{Span, TokenStream};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Expr, Ident, Path, Type};

use raw::TaskValue;

pub mod check;
mod raw;

/// The result of parsing and checking
pub type Result<T> = std::result::Result<T, Error>;

/// `app! { .. }`
pub struct App {
    /// `device: $path`
    pub device: Path,
    /// `idle: { $Idle }`
    pub idle: Option<Idle>,
    /// `init: { $Init }`
    pub init: Option<Init>,
    /// `resources: $Statics`
    pub resources: Option<Spanned<Statics>>,
    /// `tasks: { $Tasks }`
    pub tasks: Option<Spanned<Tasks>>,
    _extensible: (),
}

impl App {
    /// Parses the contents of the `app! { .. }` macro
    pub fn parse(ts: proc_macro::TokenStream) -> Result<Self> {
        let app: raw::App = syn::parse(ts)?;

        let mut device = None;
        let mut idle = None;
        let mut init = None;
        let mut resources = None;
        let mut tasks = None;

        let mut outcome = Outcome::default();
        for kv in app.kvs.into_pairs().map(|pair| pair.into_value()) {
            let key = kv.key.as_ref();
            let span = kv.key.span();
            match key {
                "device" => if let Either::Left(path) = kv.value {
                    if device.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        device = Some(path);
                    }
                } else {
                    outcome.error_value_is_not_a_path(span);
                },
                "resources" => if let Either::Right(ts) = kv.value {
                    if resources.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        let statics = syn::parse2(ts)?;
                        resources = Some(Spanned {
                            node: statics,
                            span,
                        });
                    }
                } else {
                    outcome.error_value_is_not_a_map(span);
                },
                "init" => if let Either::Right(ts) = kv.value {
                    if init.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        // A bit of hack but Idle has the exact same fields as Init so we reuse its
                        // parser here
                        let idle = Idle::parse(ts)?;
                        init = Some(Init {
                            path: idle.path,
                            resources: idle.resources,
                            _extensible: (),
                        })
                    }
                } else {
                    outcome.error_value_is_not_a_map(span);
                },
                "idle" => if let Either::Right(ts) = kv.value {
                    if idle.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        idle = Some(Idle::parse(ts)?);
                    }
                } else {
                    outcome.error_value_is_not_a_map(span);
                },
                "tasks" => if let Either::Right(ts) = kv.value {
                    if tasks.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        tasks = Some(Spanned {
                            node: Tasks::parse(ts)?,
                            span,
                        });
                    }
                } else {
                    outcome.error_value_is_not_a_map(span);
                },
                _ => outcome.error_unknown_key(span),
            }
        }

        // mandatory
        let device = device.ok_or_else(|| format_err!("`device` is missing or invalid"))?;

        if outcome.is_error() {
            Err(format_err!("Syntax error"))
        } else {
            Ok(App {
                device,
                idle,
                init,
                resources,
                tasks,
                _extensible: (),
            })
        }
    }
}

/// `static $ident: $ty [= $expr];`
pub struct Static {
    /// `$ident`
    pub ident: Ident,
    /// `$ty`
    pub ty: Type,
    /// `$expr`
    pub expr: Option<Expr>,
    _extensible: (),
}

#[doc(hidden)]
impl Synom for Static {
    named!(parse -> Self, do_parse!(
        _static: syn!(Token![static]) >>
            ident: syn!(Ident) >>
            _colon: punct!(:) >>
            ty: syn!(Type) >>
            expr: option!(do_parse!(
                _equal: punct!(=) >>
                    expr: syn!(Expr) >>
                    (expr)
            )) >>
            _semicolon: punct!(;) >>
            (Static {
                ident,
                ty,
                expr,
                _extensible: (),
            })
    ));
}

/// `$(static $ident: $ty [= $expr];)*`
pub struct Statics(pub(crate) Vec<Static>);

impl ops::Deref for Statics {
    type Target = [Static];

    fn deref(&self) -> &[Static] {
        &self.0
    }
}

#[doc(hidden)]
impl Synom for Statics {
    named!(parse -> Self, map!(many0!(syn!(Static)), Statics));
}

/// `init: { .. }`
pub struct Init {
    /// `path: $Path`
    pub path: Option<Path>,
    /// `resources: $Idents`
    pub resources: Option<Spanned<Idents>>,
    _extensible: (),
}

/// `idle: { .. }`
pub struct Idle {
    /// `path: $Path`
    pub path: Option<Path>,
    /// `resources: $Idents`
    pub resources: Option<Spanned<Idents>>,
    _extensible: (),
}

impl Idle {
    fn parse(ts: TokenStream) -> Result<Idle> {
        let idle: raw::Idle = syn::parse2(ts)?;

        let mut path = None;
        let mut resources = None;
        let mut outcome = Outcome::default();
        for kv in idle.kvs.into_pairs().map(|p| p.into_value()) {
            let span = kv.key.span();
            match kv.key.as_ref() {
                "path" => if let Either::Left(path_) = kv.value {
                    if path.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        path = Some(path_);
                    }
                } else {
                    outcome.error_value_is_not_a_path(span);
                },
                "resources" => if let Either::Right(ts) = kv.value {
                    if resources.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        let idents = syn::parse2(ts)?;
                        resources = Some(Spanned { node: idents, span });
                    }
                } else {
                    outcome.error_value_is_not_an_array(span);
                },
                _ => outcome.error_unknown_key(span),
            }
        }

        if outcome.is_error() {
            Err(format_err!("Syntax error"))
        } else {
            Ok(Idle {
                path,
                resources,
                _extensible: (),
            })
        }
    }
}

/// `$($Ident: { .. },)*`
pub struct Tasks(Vec<(Ident, Task)>);

impl Tasks {
    fn parse(ts: TokenStream) -> Result<Self> {
        let tasks: raw::Tasks = syn::parse2(ts)?;

        Ok(Tasks(tasks
            .kvs
            .into_iter()
            .map(|kv| Ok((kv.key, Task::parse(kv.value)?)))
            .collect::<Result<Vec<_>>>()?))
    }
}

/// `$Ident: { .. }`
pub struct Task {
    /// `enabled: $bool`
    pub enabled: Option<bool>,
    /// `path: $Path`
    pub path: Path,
    /// `priority: $u8`
    pub priority: Option<u8>,
    /// `resources: $Resources`
    pub resources: Option<Spanned<Idents>>,
    _extensible: (),
}

impl Task {
    fn parse(ts: TokenStream) -> Result<Self> {
        let task: raw::Task = syn::parse2(ts)?;

        let mut enabled = None;
        let mut path = None;
        let mut priority = None;
        let mut resources = None;

        let mut outcome = Outcome::default();
        for kv in task.kvs.into_pairs().map(|pair| pair.into_value()) {
            let key = kv.key.as_ref();
            let span = kv.key.span();
            match key {
                "enabled" => if let TaskValue::Bool(b) = kv.value {
                    if enabled.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        enabled = Some(b.value);
                    }
                } else {
                    outcome.error_value_is_not_a_boolean(span);
                },
                "path" => if let TaskValue::Path(p) = kv.value {
                    if path.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        path = Some(p);
                    }
                } else {
                    outcome.error_value_is_not_a_path(span);
                },
                "priority" => if let TaskValue::Int(p) = kv.value {
                    if priority.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        priority = Some(p.value().try_into()?);
                    }
                } else {
                    outcome.error_value_is_not_a_path(span);
                },
                "resources" => if let TaskValue::Idents(ts) = kv.value {
                    if resources.is_some() {
                        outcome.error_duplicate_key(span);
                    } else {
                        resources = Some(Spanned {
                            node: syn::parse2(ts)?,
                            span,
                        });
                    }
                } else {
                    outcome.error_value_is_not_a_path(span);
                },
                _ => outcome.error_unknown_key(span),
            }
        }

        let path = path.ok_or_else(|| format_err!("`path` is missing or invalid"))?;

        if outcome.is_error() {
            Err(format_err!("Syntax error"))
        } else {
            Ok(Task {
                enabled,
                path,
                priority,
                resources,
                _extensible: (),
            })
        }
    }
}

#[derive(Default)]
struct Outcome {
    error: bool,
}

impl Outcome {
    fn error(&mut self, span: Span, msg: &str) {
        self.error = true;

        span.unstable().error(msg).emit();
    }

    fn error_duplicate_key(&mut self, span: Span) {
        self.error(span, "this key appears more than once in this scope")
    }

    fn error_duplicate_resource(&mut self, span: Span) {
        self.error(
            span,
            "this resource name appears more than once in this list",
        )
    }

    fn error_duplicate_task(&mut self, span: Span) {
        self.error(span, "this task name appears more than once in this list")
    }

    fn error_owned_resource(&mut self, span: Span) {
        self.error(span, "this resource is owned by `init` and can't be shared")
    }

    fn error_undeclared_resource(&mut self, span: Span) {
        self.error(
            span,
            "this resource has not been declared in the top level list",
        )
    }

    fn error_uninitialized_resource(&mut self, span: Span) {
        self.error(
            span,
            "this resource has not been initialized in the top level list",
        )
    }

    fn error_unknown_key(&mut self, key: Span) {
        self.error(key, "unknown key")
    }

    fn error_value_is_not_an_array(&mut self, key: Span) {
        self.error(
            key,
            "the value of this key must be an array (e.g. `[A, B, C]`)",
        )
    }

    fn error_value_is_not_a_boolean(&mut self, key: Span) {
        self.error(key, "the value of this key must be a boolean (e.g. `true`)")
    }

    fn error_value_is_not_a_map(&mut self, key: Span) {
        self.error(
            key,
            "the value of this key must be a map (e.g. `{ key: value, .. }`)",
        )
    }

    fn error_value_is_not_a_path(&mut self, key: Span) {
        self.error(
            key,
            "the value of this key must be a path (e.g. `foo::bar` or `baz`)",
        )
    }

    fn warn(&self, span: Span, msg: &str) {
        span.unstable().warning(msg).emit();
    }

    fn warn_empty_list(&self, span: Span) {
        self.warn(span, "this list is empty; consider removing it");
    }

    fn is_error(&self) -> bool {
        self.error
    }
}

/// `$($Ident),*`
pub struct Idents(Vec<Ident>);

impl ops::Deref for Idents {
    type Target = [Ident];

    fn deref(&self) -> &[Ident] {
        &self.0
    }
}

impl Synom for Idents {
    named!(parse -> Self,
           map!(call!(Punctuated::<_, Token![,]>::parse_terminated),
                |p| Idents(p.into_pairs().map(|p| p.into_value()).collect())));
}

/// Value node with span information
pub struct Spanned<T> {
    node: T,
    span: Span,
}
