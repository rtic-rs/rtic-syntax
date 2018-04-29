//! Parser of the `app!` macro used by the Real Time For the Masses (RTFM) framework

#![deny(missing_docs)]
// #![deny(warnings)]
#![allow(warnings)]
#![feature(proc_macro)]

extern crate either;
#[macro_use]
extern crate failure;
extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
#[macro_use]
extern crate syn;

use std::ops::{self, Range};

pub use failure::Error;

use proc_macro2::{Span, TokenStream};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Expr, Ident, LitInt, Path, Type};

pub mod check;
mod raw;

/// The result of parsing and checking
pub type Result<T> = std::result::Result<T, Error>;

/// `app! { .. }`
pub struct App {
    /// `device: $path`
    pub device: Path,
    /// `resources: $Statics`
    pub resources: Option<Spanned<Statics>>,
    /// `idle: { $Idle }`
    pub idle: Option<Spanned<Idle>>,
    /// `init: { $Init }`
    pub init: Option<Spanned<Init>>,
    /// `free_interrupts: $Idents`
    pub free_interrupts: Option<Spanned<Idents>>,
    /// `tasks: { $Tasks }`
    pub tasks: Option<Spanned<Tasks>>,
    _extensible: (),
}

impl App {
    /// Parses the contents of the `app! { .. }` macro
    pub fn parse(ts: proc_macro::TokenStream) -> Result<Self> {
        use raw::AppValue as V;

        let app: raw::App = syn::parse(ts)?;

        let mut device = None;
        let mut free_interrupts = None;
        let mut idle = None;
        let mut init = None;
        let mut resources = None;
        let mut tasks = None;

        let mut outcome = Outcome::default();
        for kv in app.kvs.into_pairs().map(|pair| pair.into_value()) {
            let kspan = kv.key.span();

            match kv.value {
                V::Device(path) => {
                    if device.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        device = Some(path);
                    }
                }
                V::FreeInterrupts((bracket, ts)) => {
                    if free_interrupts.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        let idents = syn::parse2(ts)?;
                        free_interrupts = Some(Spanned {
                            node: idents,
                            span: bracket.0,
                        });
                    }
                }
                V::Resources((brace, ts)) => {
                    if resources.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        let statics = syn::parse2(ts)?;
                        resources = Some(Spanned {
                            node: statics,
                            span: brace.0,
                        });
                    }
                }
                V::Init((brace, ts)) => {
                    if init.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        init = Some(Spanned {
                            node: Init::parse(ts, false)?,
                            span: brace.0,
                        });
                    }
                }
                V::Idle((brace, ts)) => {
                    if idle.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        // A bit of hack but Init is a superset of Idle so we reuse its parser here
                        let init = Init::parse(ts, true)?;
                        idle = Some(Spanned {
                            node: Idle {
                                path: init.path,
                                resources: init.resources,
                                _extensible: (),
                            },
                            span: brace.0,
                        })
                    }
                }
                V::Tasks((brace, ts)) => {
                    if tasks.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        tasks = Some(Spanned {
                            node: Tasks::parse(ts)?,
                            span: brace.0,
                        });
                    }
                }
                V::Unknown(..) => outcome.error_unknown_key(kspan),
            }
        }

        // mandatory
        let device = device.ok_or_else(|| format_err!("`device` is missing or invalid"))?;

        if outcome.is_error() {
            Err(format_err!("Syntax error"))
        } else {
            Ok(App {
                device,
                free_interrupts,
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
    /// `async: $Idents`
    pub async: Option<Spanned<Idents>>,
    /// `async_after: $Idents`
    pub async_after: Option<Spanned<Idents>>,
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

impl Init {
    // Parser shared between `Idle` and `Init`
    fn parse(ts: TokenStream, is_idle: bool) -> Result<Init> {
        use raw::InitValue as V;

        let init: raw::Init = syn::parse2(ts)?;

        let mut async = None;
        let mut async_after = None;
        let mut path = None;
        let mut resources = None;
        let mut outcome = Outcome::default();
        for kv in init.kvs.into_pairs().map(|p| p.into_value()) {
            let kspan = kv.key.span();

            match kv.value {
                V::Path(p) => {
                    if path.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        path = Some(p);
                    }
                }
                V::Resources((bracket, ts)) => {
                    if resources.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        let idents = syn::parse2(ts)?;
                        resources = Some(Spanned {
                            node: idents,
                            span: bracket.0,
                        });
                    }
                }
                V::Async((bracket, ts)) => {
                    if is_idle {
                        outcome.error_unknown_key(kspan)
                    } else {
                        if async.is_some() {
                            outcome.error_duplicate_key(kspan);
                        } else {
                            let idents = syn::parse2(ts)?;
                            async = Some(Spanned {
                                node: idents,
                                span: bracket.0,
                            });
                        }
                    }
                }
                V::AsyncAfter((bracket, ts)) => {
                    if is_idle {
                        outcome.error_unknown_key(kspan)
                    } else {
                        if async_after.is_some() {
                            outcome.error_duplicate_key(kspan);
                        } else {
                            let idents = syn::parse2(ts)?;
                            async_after = Some(Spanned {
                                node: idents,
                                span: bracket.0,
                            });
                        }
                    }
                }
                V::Unknown(..) => outcome.error_unknown_key(kspan),
            }
        }

        if outcome.is_error() {
            Err(format_err!("Syntax error"))
        } else {
            Ok(Init {
                path,
                resources,
                async,
                async_after,
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
    /// `interrupt: $Ident`
    pub interrupt: Option<Ident>,
    /// `path: $Path`
    pub path: Option<Path>,
    /// `input: $Type`
    pub input: Option<Type>,
    /// `priority: $LitInt`
    pub priority: Option<LitInt>,
    /// `capacity: $LitInt`
    pub capacity: Option<LitInt>,
    /// `resources: $Resources`
    pub resources: Option<Spanned<Idents>>,
    /// `async: $Idents`
    pub async: Option<Spanned<Idents>>,
    /// `async_after: $Idents`
    pub async_after: Option<Spanned<Idents>>,
    _extensible: (),
}

impl Task {
    fn parse(ts: TokenStream) -> Result<Self> {
        use raw::TaskValue as V;

        let task: raw::Task = syn::parse2(ts)?;

        let mut async = None;
        let mut async_after = None;
        let mut capacity = None;
        let mut input = None;
        let mut interrupt = None;
        let mut path = None;
        let mut priority = None;
        let mut resources = None;

        let mut outcome = Outcome::default();
        for kv in task.kvs.into_pairs().map(|pair| pair.into_value()) {
            let kspan = kv.key.span();

            match kv.value {
                V::Interrupt(id) => {
                    if interrupt.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        interrupt = Some(id);
                    }
                }
                V::Path(p) => {
                    if path.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        path = Some(p);
                    }
                }
                V::Input(ty) => {
                    if input.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        input = Some(ty);
                    }
                }
                V::Async((bracket, ts)) => {
                    if async.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        async = Some(Spanned {
                            node: syn::parse2(ts)?,
                            span: bracket.0,
                        });
                    }
                }
                V::AsyncAfter((bracket, ts)) => {
                    if async_after.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        async_after = Some(Spanned {
                            node: syn::parse2(ts)?,
                            span: bracket.0,
                        });
                    }
                }
                V::Capacity(lit) => {
                    if capacity.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        capacity = Some(lit);
                    }
                }
                V::Priority(lit) => {
                    if priority.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        priority = Some(lit);
                    }
                }
                V::Resources((bracket, ts)) => {
                    if resources.is_some() {
                        outcome.error_duplicate_key(kspan);
                    } else {
                        resources = Some(Spanned {
                            node: syn::parse2(ts)?,
                            span: bracket.0,
                        });
                    }
                }
                V::Unknown(..) => outcome.error_unknown_key(kspan),
            }
        }

        if outcome.is_error() {
            Err(format_err!("Syntax error"))
        } else {
            Ok(Task {
                async,
                async_after,
                capacity,
                input,
                interrupt,
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

    fn error_duplicate_async(&mut self, span: Span) {
        self.error(
            span,
            "a task can't be listed under `async` and `async_after` at the same time",
        )
    }

    fn error_free_interrupt_isnt(&mut self, span: Span) {
        self.error(span, "this interrupt is bound to a task");
    }

    fn error_interrupt_task_with_input(&mut self, span: Span) {
        self.error(span, "task bound to interrupt must have no input");
    }

    fn error_invalid_async(&mut self, span: Span) {
        self.error(
            span,
            "Tasks bound to interrupts can't be asynchronously called",
        )
    }

    fn error_out_of_range(&mut self, span: Span, range: Range<u64>) {
        self.error(
            span,
            &format!(
                "this value is outside the valid range of `{:?}`",
                (range.start, range.end)
            ),
        )
    }

    fn error_owned_resource(&mut self, span: Span) {
        self.error(span, "this resource is owned by `init` and can't be shared")
    }

    fn error_shared_interrupt(&mut self, span: Span) {
        self.error(span, "this interrupt is already bound to another task")
    }

    fn error_undeclared_resource(&mut self, span: Span) {
        self.error(
            span,
            "this resource has not been declared in the top level list",
        )
    }

    fn error_undeclared_task(&mut self, span: Span) {
        self.error(
            span,
            "this task has not been declared in the top level list",
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

    fn warn(&self, span: Span, msg: &str) {
        span.unstable().warning(msg).emit();
    }

    fn warn_default_value(&self, span: Span) {
        self.warn(
            span,
            "this is the default value; consider removing this key value pair",
        );
    }

    fn warn_empty_list(&self, span: Span) {
        self.warn(span, "this list is empty; consider removing it");
    }

    fn warn_unused_task(&self, span: Span) {
        self.warn(
            span,
            "this task is not bound to an interrupt, or asynchronously called",
        );
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
