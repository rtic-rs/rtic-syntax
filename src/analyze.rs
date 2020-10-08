//! RTIC application analysis

use core::cmp;
use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use syn::{Ident, Type};

use crate::{ast::App, Set};

pub(crate) fn app(app: &App) -> Analysis {
    // a. Initialization of resources
    let mut late_resources = LateResources::new();
    if !app.late_resources.is_empty() {
        let mut resources = app.late_resources.keys().cloned().collect::<BTreeSet<_>>();
        let mut rest = false;
        if let Some(init) = &app.inits.first() {
            if init.args.late.is_empty() {
                rest = true;
            } else {
                let mut late_resources = Vec::new();

                for name in &init.args.late {
                    late_resources.push(name.clone());
                    resources.remove(name);
                }
            }
        }

        if rest {
            late_resources.push(resources);
        }
    }

    // e. Location of resources
    let mut locations = app
        .late_resources
        .iter()
        .chain(app.resources.iter().map(|(name, res)| (name, &res.late)))
        .filter_map(|(_name, _lr)| None)
        .collect::<Locations>();

    let mut ownerships = Ownerships::new();
    let mut sync_types = SyncTypes::new();
    for (prio, name, access) in app.resource_accesses() {
        let res = app.resource(name).expect("UNREACHABLE").0;

        // (e)
        // Add each resource to locations
        locations.insert(name.clone(), Location::Owned);

        // (c)
        if let Some(priority) = prio {
            if let Some(ownership) = ownerships.get_mut(name) {
                match *ownership {
                    Ownership::Owned { priority: ceiling }
                    | Ownership::CoOwned { priority: ceiling }
                    | Ownership::Contended { ceiling }
                        if priority != ceiling =>
                    {
                        *ownership = Ownership::Contended {
                            ceiling: cmp::max(ceiling, priority),
                        };

                        if access.is_shared() {
                            sync_types.insert(res.ty.clone());
                        }
                    }

                    Ownership::Owned { priority: ceil } if ceil == priority => {
                        *ownership = Ownership::CoOwned { priority };
                    }

                    _ => {}
                }
            } else {
                ownerships.insert(name.clone(), Ownership::Owned { priority });
            }
        }
    }

    // Most late resources need to be `Send`
    let mut send_types = SendTypes::new();
    let owned_by_idle = Ownership::Owned { priority: 0 };
    for (name, res) in app.late_resources.iter() {
        // handle not owned by idle
        if ownerships
            .get(name)
            .map(|ownership| *ownership != owned_by_idle)
            .unwrap_or(false)
        {
            send_types.insert(res.ty.clone());
        }
    }

    // All resources shared with `init` (ownership != None) need to be `Send`
    for name in app.inits.iter().flat_map(|init| init.args.resources.keys()) {
        if let Some(ownership) = ownerships.get(name) {
            if *ownership != owned_by_idle {
                send_types.insert(app.resources[name].ty.clone());
            }
        }
    }

    let mut channels = Channels::new();

    for (name, spawnee) in &app.software_tasks {
        let spawnee_prio = spawnee.args.priority;

        let channel = channels.entry(spawnee_prio).or_default();
        channel.tasks.insert(name.clone());

        // All inputs are now send as we do not know from where they may be spawned.
        spawnee.inputs.iter().for_each(|input| {
            send_types.insert(input.ty.clone());
        });
    }

    // No channel should ever be empty
    debug_assert!(channels.values().all(|channel| !channel.tasks.is_empty()));

    // Compute channel capacities
    for channel in channels.values_mut() {
        channel.capacity = channel
            .tasks
            .iter()
            .map(|name| app.software_tasks[name].args.capacity)
            .sum();
    }

    Analysis {
        channels,
        late_resources,
        locations,
        ownerships,
        send_types,
        sync_types,
    }
}

/// Priority ceiling
pub type Ceiling = Option<u8>;

/// Task priority
pub type Priority = u8;

/// Resource name
pub type Resource = Ident;

/// Task name
pub type Task = Ident;

/// The result of analyzing an RTIC application
pub struct Analysis {
    /// SPSC message channels
    pub channels: Channels,

    /// The late resources
    pub late_resources: LateResources,

    /// Location of all *used* resources
    ///
    /// If a resource is not listed here it means that's a "dead" (never accessed) resource and the
    /// backend should not generate code for it
    ///
    /// `None` indicates that the resource must reside in shared memory
    pub locations: Locations,

    /// Resource ownership
    pub ownerships: Ownerships,

    /// These types must implement the `Send` trait
    pub send_types: SendTypes,

    /// These types must implement the `Sync` trait
    pub sync_types: SyncTypes,
}

/// All channels, keyed by dispatch priority
pub type Channels = BTreeMap<Priority, Channel>;

/// Late resources, wrapped in a vector
pub type LateResources = Vec<BTreeSet<Resource>>;

/// Location of all *used* resources
pub type Locations = IndexMap<Resource, Location>;

/// Resource ownership
pub type Ownerships = IndexMap<Resource, Ownership>;

/// These types must implement the `Send` trait
pub type SendTypes = Set<Box<Type>>;

/// These types must implement the `Sync` trait
pub type SyncTypes = Set<Box<Type>>;

/// A channel used to send messages
#[derive(Debug, Default)]
pub struct Channel {
    /// The channel capacity
    pub capacity: u8,

    /// Tasks that can be spawned on this channel
    pub tasks: BTreeSet<Task>,
}

/// Resource ownership
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ownership {
    /// Owned by a single task
    Owned {
        /// Priority of the task that owns this resource
        priority: u8,
    },

    /// "Co-owned" by more than one task; all of them have the same priority
    CoOwned {
        /// Priority of the tasks that co-own this resource
        priority: u8,
    },

    /// Contended by more than one task; the tasks have different priorities
    Contended {
        /// Priority ceiling
        ceiling: u8,
    },
}

impl Ownership {
    /// Whether this resource needs to a lock at this priority level
    pub fn needs_lock(&self, priority: u8) -> bool {
        match self {
            Ownership::Owned { .. } | Ownership::CoOwned { .. } => false,

            Ownership::Contended { ceiling } => {
                debug_assert!(*ceiling >= priority);

                priority < *ceiling
            }
        }
    }

    /// Whether this resource is exclusively owned
    pub fn is_owned(&self) -> bool {
        match self {
            Ownership::Owned { .. } => true,
            _ => false,
        }
    }
}

/// Resource location
#[derive(Clone, Debug, PartialEq)]
pub enum Location {
    /// resource that is owned
    Owned,
}
