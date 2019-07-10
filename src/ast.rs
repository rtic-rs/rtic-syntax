//! Abstract Syntax Tree

use core::ops::Deref;
use std::collections::BTreeMap;

use syn::{ArgCaptured, Attribute, Expr, Ident, Pat, Path, Stmt, Type};

use crate::{Core, Map, Set};

/// The `#[app]` attribute
#[derive(Debug)]
pub struct App {
    /// The arguments to the `#[app]` attribute
    pub args: AppArgs,

    /// The name of the `const` item on which the `#[app]` attribute has been placed
    pub name: Ident,

    // NOTE one per core
    /// Per-core `#[init]` functions
    pub inits: Inits,

    /// Per-core `#[idle]` functions
    pub idles: Idles,

    /// Late (runtime initialized) resources
    pub late_resources: Map<LateResource>,

    /// Early (compile time initialized) resources
    pub resources: Map<Resource>,

    /// Hardware tasks: `#[task(binds = ..)]`s
    pub hardware_tasks: Map<HardwareTask>,

    /// Software tasks: `#[task]`
    pub software_tasks: Map<SoftwareTask>,

    /// Interrupts used to dispatch software tasks
    pub extern_interrupts: ExternInterrupts,

    pub(crate) _extensible: (),
}

/// Interrupts used to dispatch software tasks
pub type ExternInterrupts = BTreeMap<Core, Map<ExternInterrupt>>;

/// The arguments of the `#[app]` attribute
#[derive(Debug)]
pub struct AppArgs {
    /// The number of cores the application will use
    pub cores: u8,

    /// Custom arguments
    pub custom: Map<CustomArg>,
}

/// A custom argument
#[derive(Debug)]
pub enum CustomArg {
    /// A boolean: `true` or `false`
    Bool(bool),

    /// An unsigned integer
    UInt(u64),

    /// An item path
    Path(Path),
}

/// Per-core `init` functions
pub type Inits = BTreeMap<u8, Init>;
/// Per-core `idle` functions
pub type Idles = BTreeMap<u8, Idle>;

/// The `init`-ialization function
#[derive(Debug)]
pub struct Init {
    /// `init` context metadata
    pub args: InitArgs,

    /// Attributes that will apply to this `init` function
    pub attrs: Vec<Attribute>,

    /// The name of the `#[init]` function
    pub name: Ident,

    /// The context argument
    pub context: Pat,

    /// Whether this `init` function returns `LateResources` or not
    pub returns_late_resources: bool,

    /// Static variables local to this context
    pub locals: Map<Local>,
    /// The statements that make up this `init` function
    pub stmts: Vec<Stmt>,

    pub(crate) _extensible: (),
}

/// `init` context metadata
#[derive(Debug, Default)]
pub struct InitArgs {
    /// Which core this context belongs to?
    pub core: u8,

    /// Late resources that will be initialized by this core
    ///
    /// NOTE do not use this field for codegen; use `Analysis.late_resources` instead
    pub late: Set<Ident>,

    /// Resources that can be accessed from this context
    pub resources: Resources,

    /// Software tasks that can be spawned from this context
    pub spawn: Set<Ident>,

    /// Software tasks that can be scheduled from this context
    pub schedule: Set<Ident>,

    pub(crate) _extensible: (),
}

/// The `idle` context
#[derive(Debug)]
pub struct Idle {
    /// `idle` context metadata
    pub args: IdleArgs,

    /// Attributes that will apply to this `idle` function
    pub attrs: Vec<Attribute>,

    /// The name of the `#[idle]` function
    pub name: Ident,

    /// The context argument
    pub context: Pat,

    /// Static variables local to this context
    pub locals: Map<Local>,

    /// The statements that make up this `idle` function
    pub stmts: Vec<Stmt>,

    pub(crate) _extensible: (),
}

/// `idle` context metadata
#[derive(Debug)]
pub struct IdleArgs {
    /// Which core this context belongs to?
    pub core: u8,

    /// Resources that can be accessed from this context
    pub resources: Resources,

    /// Software tasks that can be spawned from this context
    pub spawn: Set<Ident>,

    /// Software tasks that can be scheduled from this context
    pub schedule: Set<Ident>,

    pub(crate) _extensible: (),
}

/// An early (compile time initialized) resource
#[derive(Debug)]
pub struct Resource {
    pub(crate) late: LateResource,
    /// The initial value of this resource
    pub expr: Box<Expr>,
}

impl Deref for Resource {
    type Target = LateResource;

    fn deref(&self) -> &LateResource {
        &self.late
    }
}

/// A late (runtime initialized) resource
#[derive(Debug)]
pub struct LateResource {
    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// Attributes that will apply to this resource
    pub attrs: Vec<Attribute>,

    /// The type of this resource
    pub ty: Type,

    pub(crate) _extensible: (),
}

/// A software task
#[derive(Debug)]
pub struct SoftwareTask {
    /// Software task metadata
    pub args: SoftwareTaskArgs,

    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,
    /// Attributes that will apply to this interrupt handler
    pub attrs: Vec<Attribute>,

    /// The context argument
    pub context: Pat,
    /// The inputs of this software task
    pub inputs: Vec<ArgCaptured>,

    /// Static variables local to this context
    pub locals: Map<Local>,
    /// The statements that make up the task handler
    pub stmts: Vec<Stmt>,

    pub(crate) _extensible: (),
}

/// Software task metadata
#[derive(Debug)]
pub struct SoftwareTaskArgs {
    /// The core this task will run on
    pub core: u8,

    /// The task capacity: the maximum number of pending messages that can be queued
    pub capacity: u8,

    /// The priority of this task
    pub priority: u8,

    /// Resources that can be accessed from this context
    pub resources: Resources,

    /// Software tasks that can be spawned from this context
    pub spawn: Set<Ident>,

    /// Software tasks that can be scheduled from this context
    pub schedule: Set<Ident>,

    pub(crate) _extensible: (),
}

impl Default for SoftwareTaskArgs {
    fn default() -> Self {
        Self {
            core: 0,
            capacity: 1,
            priority: 1,
            resources: Resources::new(),
            spawn: Set::new(),
            schedule: Set::new(),
            _extensible: (),
        }
    }
}

/// A hardware task
#[derive(Debug)]
pub struct HardwareTask {
    /// Hardware task metadata
    pub args: HardwareTaskArgs,

    /// Attributes that will apply to this interrupt handler
    pub attrs: Vec<Attribute>,

    /// The context argument
    pub context: Pat,

    /// Static variables local to this context
    pub locals: Map<Local>,
    /// The statements that make up the task handler
    pub stmts: Vec<Stmt>,

    pub(crate) _extensible: (),
}

/// Hardware task metadata
#[derive(Debug)]
pub struct HardwareTaskArgs {
    /// The core on which this task will be executed
    pub core: u8,

    /// The interrupt or exception that this task is bound to
    pub binds: Ident,

    /// The priority of this task
    pub priority: u8,

    /// Resources that can be accessed from this context
    pub resources: Resources,

    /// Software tasks that can be spawned from this context
    pub spawn: Set<Ident>,

    /// Software tasks that can be scheduled from this context
    pub schedule: Set<Ident>,

    pub(crate) _extensible: (),
}

/// Interrupt that could be used to dispatch software tasks
#[derive(Debug)]
pub struct ExternInterrupt {
    /// Attributes that will apply to this interrupt handler
    pub attrs: Vec<Attribute>,

    pub(crate) _extensible: (),
}

/// A `static mut` variable local to and owned by a context
#[derive(Debug)]
pub struct Local {
    /// Attributes like `#[link_section]`
    pub attrs: Vec<Attribute>,

    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// Type
    pub ty: Box<Type>,

    /// Initial value
    pub expr: Box<Expr>,

    pub(crate) _extensible: (),
}

/// Resource access
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Access {
    /// `[x]`
    Exclusive,

    /// `[&x]`
    Shared,
}

impl Access {
    /// Is this enum in the `Exclusive` variant?
    pub fn is_exclusive(&self) -> bool {
        *self == Access::Exclusive
    }

    /// Is this enum in the `Shared` variant?
    pub fn is_shared(&self) -> bool {
        *self == Access::Shared
    }
}

/// Resource access list
pub type Resources = Map<Access>;
