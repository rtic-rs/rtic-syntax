//! Specification checker

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use either::Either;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _Spanned;
use syn::token::Paren;
use syn::{self, Expr, Ident, LitInt, Path, PathSegment, Type, TypeTuple};

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
    /// `free_interrupts: $Idents`
    pub free_interrupts: Idents,
    /// `tasks: { $Tasks }`
    pub tasks: Tasks,
    _extensible: (),
}

impl super::App {
    /// Checks the parsed specification
    pub fn check(self) -> Result<App> {
        let mut outcome = Outcome::default();

        let resources = if let Some(resources) = self.resources {
            resources.check(&mut outcome)
        } else {
            Statics::new()
        };

        let init = if let Some(init) = self.init {
            init.check(&resources, &mut outcome)
        } else {
            Init::default()
        };

        let idle = if let Some(idle) = self.idle {
            idle.check(&resources, &init, &mut outcome)
        } else {
            Idle::default()
        };

        let free_interrupts = check::idents(self.free_interrupts, &mut outcome);

        let tasks = if let Some(tasks) = self.tasks {
            tasks.check(&resources, &init, &mut outcome)
        } else {
            Tasks::new()
        };

        check::tasks(&init, &tasks, &free_interrupts, &mut outcome);

        if outcome.is_error() {
            Err(format_err!("Specification error"))
        } else {
            Ok(App {
                device: self.device,
                idle,
                free_interrupts,
                tasks,
                init,
                resources,
                _extensible: (),
            })
        }
    }
}

/// Checked [`Init`](../struct.Init.html)
pub struct Init {
    /// `path: $Path`
    pub path: Path,
    /// `resources: $Resources`
    pub resources: Idents,
    /// `schedule_now: $Idents`
    pub schedule_now: Idents,
    /// `schedule_after: $Idents`
    pub schedule_after: Idents,
    _extensible: (),
}

impl Default for Init {
    fn default() -> Self {
        Init {
            path: mkpath("init"),
            resources: Idents::new(),
            schedule_now: Idents::new(),
            schedule_after: Idents::new(),
            _extensible: (),
        }
    }
}

impl Spanned<super::Init> {
    fn check(self, statics: &Statics, outcome: &mut Outcome) -> Init {
        let path = check::path(self.node.path, "init", &outcome);

        let resources = check::resources(self.node.resources, statics, None, outcome);

        let schedule_now = check::idents(self.node.schedule_now, outcome);

        let schedule_after = check::idents(self.node.schedule_after, outcome);

        Init {
            path,
            resources,
            schedule_now,
            schedule_after,
            _extensible: (),
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
            path: mkpath("idle"),
            resources: Idents::new(),
            _extensible: (),
        }
    }
}

impl Spanned<super::Idle> {
    fn check(self, statics: &Statics, init: &Init, outcome: &mut Outcome) -> Idle {
        let path = check::path(self.node.path, "idle", &outcome);

        let resources = check::resources(self.node.resources, statics, Some(init), outcome);

        Idle {
            path,
            resources,
            _extensible: (),
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
    fn check(self, outcome: &mut Outcome) -> Statics {
        let mut resources = Statics::new();

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

        resources
    }
}

/// The RHS part (`: { .. }`) of a task
pub struct Task {
    /// `interrupt: $Ident` || `instances: $u8`
    pub interrupt_or_instances: Either<Ident, u8>,
    /// `path: $Path`
    pub path: Path,
    /// `input: $Type` - `None` means no input, i.e. the input type is `()`
    pub input: Option<Type>,
    /// `priority: $u8`
    pub priority: u8,
    /// `resources: $Resources`
    pub resources: Idents,
    /// `schedule_now: $Idents`
    pub schedule_now: Idents,
    /// `schedule_after: $Idents`
    pub schedule_after: Idents,
    _extensible: (),
}

impl Spanned<super::Tasks> {
    fn check(self, statics: &Statics, init: &Init, outcome: &mut Outcome) -> Tasks {
        let mut tasks = Tasks::new();

        if self.node.0.is_empty() {
            outcome.warn_empty_list(self.span)
        }

        for (name, task) in self.node.0 {
            if tasks.contains_key(&name) {
                outcome.error_duplicate_task(name.span());
                continue;
            }

            tasks.insert(name, task.check(&name, statics, init, outcome));
        }

        tasks
    }
}

impl super::Task {
    fn check(self, name: &Ident, statics: &Statics, init: &Init, outcome: &mut Outcome) -> Task {
        let interrupt_or_instances = match (self.interrupt, self.instances) {
            (Some(interrupt), Some(instances)) => {
                outcome.error(
                    instances.span(),
                    "`instances` and `interrupt` can't be specified at the same time",
                );

                Either::Left(interrupt)
            }
            (Some(interrupt), None) => Either::Left(interrupt),
            (None, Some(instances)) => {
                Either::Right(check::lit_int(Some(instances), 1, 1..255, outcome) as u8)
            }
            (None, None) => Either::Right(1),
        };

        let input = check::input(self.input, outcome);

        if interrupt_or_instances.is_left() && input.is_some() {
            outcome.error_event_task_with_input(input.span());
        }

        Task {
            interrupt_or_instances,
            path: check::path(self.path, name.as_ref(), outcome),
            priority: check::lit_int(self.priority, 1, 1..255, outcome) as u8,
            input,
            resources: check::resources(self.resources, statics, Some(init), outcome),
            schedule_now: check::idents(self.schedule_now, outcome),
            schedule_after: check::idents(self.schedule_after, outcome),
            _extensible: (),
        }
    }
}

fn idents(idents: Option<Spanned<super::Idents>>, outcome: &mut Outcome) -> Idents {
    if let Some(idents) = idents.as_ref() {
        if idents.node.is_empty() {
            outcome.warn_empty_list(idents.span)
        }
    }

    let mut set = Idents::new();
    for ident in idents.map(|ids| ids.node.0).unwrap_or(vec![]) {
        if set.contains(&ident) {
            outcome.error_duplicate_resource(ident.span());
            continue;
        }

        set.insert(ident);
    }

    set
}

fn input(input: Option<Type>, outcome: &mut Outcome) -> Option<Type> {
    let def = mkunit();
    if let Some(input) = input.as_ref() {
        if input == &def {
            outcome.warn_default_value(input.span());
            return None;
        }
    }

    input
}

fn lit_int(lit: Option<LitInt>, def: u64, range: Range<u64>, outcome: &mut Outcome) -> u64 {
    if let Some(lit) = lit.as_ref() {
        if lit.value() == def {
            outcome.warn_default_value(lit.span());
        }
    }

    let val = lit.as_ref().map(|lit| lit.value()).unwrap_or(def);

    if val < range.start || val > range.end {
        outcome.error_out_of_range(lit.span(), range);
    }

    val
}

fn path(path: Option<Path>, def: &str, outcome: &Outcome) -> Path {
    let def_path = syn::parse_str(def).unwrap();
    if let Some(path) = path.as_ref() {
        if path == &def_path {
            outcome.warn_default_value(path.span());
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

        if statics.get(&ident).is_none() {
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

fn tasks(init: &Init, tasks: &Tasks, free_interrupts: &Idents, outcome: &mut Outcome) {
    let mut callable = HashSet::new();
    let mut noncallable = HashSet::new();
    let mut interrupts = HashSet::new();

    for (name, task) in tasks {
        if let Either::Left(int) = task.interrupt_or_instances.as_ref() {
            noncallable.insert(name);

            if interrupts.contains(int) {
                outcome.error_shared_interrupt(int.span());
                continue;
            }

            interrupts.insert(int);
        } else {
            callable.insert(name);
        }
    }

    let mut unused = callable.clone();

    // check `init`
    for duplicate in init.schedule_now.intersection(&init.schedule_after) {
        outcome.error_duplicate_schedule(duplicate.span())
    }

    for task in init.schedule_now.iter().chain(&init.schedule_after) {
        if noncallable.contains(task) {
            outcome.error_invalid_schedule(task.span());
        } else if callable.contains(task) {
            unused.remove(task);
        } else {
            outcome.error_undeclared_task(task.span());
        }
    }

    // check tasks
    for task in tasks.values() {
        for duplicate in task.schedule_now.intersection(&task.schedule_after) {
            outcome.error_duplicate_schedule(duplicate.span())
        }

        for t in task.schedule_now.iter().chain(&task.schedule_after) {
            if noncallable.contains(t) {
                outcome.error_invalid_schedule(t.span());
            } else if callable.contains(t) {
                unused.remove(t);
            } else {
                outcome.error_undeclared_task(t.span());
            }
        }
    }

    // unused tasks
    for task in unused {
        outcome.warn_unused_task(task.span());
    }

    // free interrupts that aren't
    for int in free_interrupts {
        if interrupts.contains(int) {
            outcome.error_free_interrupt_isnt(int.span());
        }
    }
}

fn mkpath(id: &str) -> Path {
    Path::from(PathSegment::from(Ident::new(id, Span::call_site())))
}

fn mkunit() -> Type {
    Type::Tuple(TypeTuple {
        paren_token: Paren(Span::call_site()),
        elems: Punctuated::new(),
    })
}
