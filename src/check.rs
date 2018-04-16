//! Specification checker

use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use syn::spanned::Spanned as _Spanned;
use syn::{self, Expr, Ident, Path, PathSegment, Type};

use {check, Outcome, Result, Spanned};

/// Checked [`Idents`](../struct.Idents.html)
///
/// No duplicate identifier in this list
pub type Idents = HashSet<Ident>;

/// Checked [`Statics`](../struct.Statics.html)
///
/// No duplicate static in this list
pub type Statics = HashMap<Ident, Static>;

/// Checked [`Tasks`](../struct.Tasks.html)
///
/// No duplicate task in this list
pub type Tasks = HashMap<Ident, Task>;

/// Checked [`App`](../struct.App.html)
pub struct App {
    /// `device: $path`
    pub device: Path,
    /// `idle: { $Idle }`
    pub idle: Idle,
    /// `init: { $Init }`
    pub init: Init,
    /// `resources: $Statics`
    pub resources: Statics,
    /// `tasks: { $Tasks }`
    pub tasks: Tasks,
    _extensible: (),
}

impl super::App {
    /// Checks the parsed specification
    pub fn check(self) -> Result<App> {
        let resources = if let Some(resources) = self.resources {
            resources.check()?
        } else {
            Statics::new()
        };

        let init = if let Some(init) = self.init {
            init.check(&resources)?
        } else {
            Init::default()
        };

        Ok(App {
            device: self.device,
            idle: if let Some(idle) = self.idle {
                idle.check(&resources, &init)?
            } else {
                Idle::default()
            },
            tasks: if let Some(tasks) = self.tasks {
                tasks.check(&resources, &init)?
            } else {
                Tasks::new()
            },
            init,
            resources,
            _extensible: (),
        })
    }
}

/// Checked [`Init`](../struct.Init.html)
pub struct Init {
    /// `path: $Path`
    pub path: Path,
    /// `resources: $Resources`
    pub resources: Idents,
    _extensible: (),
}

impl Default for Init {
    fn default() -> Self {
        Init {
            path: Path::from(PathSegment::from(Ident::new("init", Span::call_site()))),
            resources: Idents::new(),
            _extensible: (),
        }
    }
}

impl super::Init {
    fn check(self, statics: &Statics) -> Result<Init> {
        let mut outcome = Outcome::default();

        let path = check::path(self.path, "init", &outcome);

        let resources = check::resources(self.resources, statics, None, &mut outcome);

        if outcome.is_error() {
            Err(format_err!("Specification error"))
        } else {
            Ok(Init {
                path,
                resources,
                _extensible: (),
            })
        }
    }
}

/// Checked [`Idle`](../struct.Idle.html)
pub struct Idle {
    /// `path: $Path`
    pub path: Path,
    /// `resources: $Resources`
    pub resources: Idents,
    _extensible: (),
}

impl Default for Idle {
    fn default() -> Self {
        Idle {
            path: Path::from(PathSegment::from(Ident::new("idle", Span::call_site()))),
            resources: Idents::new(),
            _extensible: (),
        }
    }
}

impl super::Idle {
    fn check(self, statics: &Statics, init: &Init) -> Result<Idle> {
        let mut outcome = Outcome::default();

        let path = check::path(self.path, "idle", &outcome);

        let resources = check::resources(self.resources, statics, Some(init), &mut outcome);

        if outcome.is_error() {
            Err(format_err!("Specification error"))
        } else {
            Ok(Idle {
                path,
                resources,
                _extensible: (),
            })
        }
    }
}

/// The `: $ty [ = $expr]` part of a static
pub struct Static {
    /// `$ty`
    pub ty: Type,
    /// `$expr`
    pub expr: Option<Expr>,
    _extensible: (),
}

impl Spanned<super::Statics> {
    fn check(self) -> Result<Statics> {
        let mut resources = Statics::new();

        let mut outcome = Outcome::default();

        if self.node.0.is_empty() {
            outcome.warn_empty_list(self.span);
        }

        for static_ in self.node.0 {
            if resources.contains_key(&static_.ident) {
                outcome.error_duplicate_resource(static_.ident.span());
            }

            resources.insert(
                static_.ident,
                Static {
                    ty: static_.ty,
                    expr: static_.expr,
                    _extensible: (),
                },
            );
        }

        if outcome.is_error() {
            Err(format_err!("Specification error"))
        } else {
            Ok(resources)
        }
    }
}

/// The RHS part (`: { .. }`) of a task
pub struct Task {
    /// `enabled: $bool`
    pub enabled: Option<bool>,
    /// `path: $Path`
    pub path: Path,
    /// `priority: $u8`
    pub priority: Option<u8>,
    /// `resources: $Resources`
    pub resources: Idents,
    _extensible: (),
}

impl Spanned<super::Tasks> {
    fn check(self, statics: &Statics, init: &Init) -> Result<Tasks> {
        let mut tasks = Tasks::new();

        let mut outcome = Outcome::default();

        if self.node.0.is_empty() {
            outcome.warn_empty_list(self.span)
        }

        for (name, task) in self.node.0 {
            if tasks.contains_key(&name) {
                outcome.error_duplicate_task(name.span());
                continue;
            }

            tasks.insert(name, task.check(statics, init, &mut outcome));
        }

        if outcome.is_error() {
            Err(format_err!("Specification error"))
        } else {
            Ok(tasks)
        }
    }
}

impl super::Task {
    fn check(self, statics: &Statics, init: &Init, outcome: &mut Outcome) -> Task {
        Task {
            enabled: self.enabled,
            path: self.path,
            priority: self.priority,
            resources: check::resources(self.resources, statics, Some(init), outcome),
            _extensible: (),
        }
    }
}

fn path(path: Option<Path>, def: &str, outcome: &Outcome) -> Path {
    let def_path = syn::parse_str(def).unwrap();
    if let Some(path) = path.as_ref() {
        if path == &def_path {
            outcome.warn(
                path.span(),
                "this is the default path; consider removing this key value pair",
            );
        }
    }

    path.unwrap_or(def_path)
}

fn resources(
    idents: Option<Spanned<super::Idents>>,
    statics: &Statics,
    init: Option<&Init>,
    outcome: &mut Outcome,
) -> Idents {
    if let Some(idents) = idents.as_ref() {
        if idents.node.is_empty() {
            outcome.warn_empty_list(idents.span)
        }
    }

    let mut resources = Idents::new();
    for ident in idents.map(|ids| ids.node.0).unwrap_or(vec![]) {
        if resources.contains(&ident) {
            outcome.error_duplicate_resource(ident.span());
            continue;
        }

        if let Some(static_) = statics.get(&ident) {
            let we_are_init = init.is_none();
            if we_are_init && static_.expr.is_none() {
                outcome.error_uninitialized_resource(ident.span());
                continue;
            }
        } else {
            outcome.error_undeclared_resource(ident.span());
            continue;
        }

        if let Some(init) = init {
            if init.resources.contains(&ident) {
                outcome.error_owned_resource(ident.span());
            }
        }

        resources.insert(ident);
    }

    resources
}
