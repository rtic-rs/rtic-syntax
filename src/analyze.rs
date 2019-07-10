//! RTFM application analysis

use core::cmp;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet, HashMap};

use indexmap::IndexMap;
use syn::{Ident, Type};

use crate::{ast::App, Core, Set};

pub(crate) fn app(app: &App) -> Analysis {
    // a. Which core initializes which resources
    let mut late_resources = LateResources::new();
    if !app.late_resources.is_empty() {
        let mut resources = app.late_resources.keys().cloned().collect::<BTreeSet<_>>();
        let mut rest = None;
        for (&core, init) in &app.inits {
            if init.args.late.is_empty() {
                // this was checked in the `check` pass
                debug_assert!(rest.is_none());

                rest = Some(core);
            } else {
                let late_resources = late_resources.entry(core).or_default();

                for name in &init.args.late {
                    late_resources.insert(name.clone());
                    resources.remove(name);
                }
            }
        }

        if let Some(rest) = rest {
            late_resources.insert(rest, resources);
        }
    }

    // c. Ceiling analysis of Exclusive resources
    // d. Sync-ness of Access::Shared resources
    // e. Location of resources
    // f. Cross initialization needs a post-initialization synchronization barrier
    let mut initialization_barriers = InitializationBarriers::new();
    let mut locations = Locations::new();
    let mut ownerships = Ownerships::new();
    let mut shared_accesses = HashMap::new();
    let mut sync_types = SyncTypes::new();
    for (core, prio, name, access) in app.resource_accesses() {
        let res = app.resource(name).expect("UNREACHABLE").0;

        // (d)
        if access.is_shared() {
            if let Some(&other_core) = shared_accesses.get(name) {
                if other_core != core {
                    // a resources accessed from different cores needs to be `Sync` regardless of
                    // priorities
                    sync_types.entry(core).or_default().insert(res.ty.clone());
                    sync_types
                        .entry(other_core)
                        .or_default()
                        .insert(res.ty.clone());
                }
            } else {
                shared_accesses.insert(name, core);
            }
        }

        // (e)
        if let Some(loc) = locations.get_mut(name) {
            match loc {
                Location::Owned {
                    core: other_core, ..
                } => {
                    if core != *other_core {
                        let mut cores = BTreeSet::new();
                        cores.insert(core);
                        cores.insert(*other_core);
                        *loc = Location::Shared { cores };
                    }
                }

                Location::Shared { cores } => {
                    cores.insert(core);
                }
            }
        } else {
            locations.insert(
                name.clone(),
                Location::Owned {
                    core,
                    cross_initialized: false,
                },
            );
        }

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
                            sync_types.entry(core).or_default().insert(res.ty.clone());
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

        // (f) in cross-initialization the initializer core is like a sender and the user core is
        // like a receiver
        let receiver = core;
        for (&sender, resources) in &late_resources {
            if sender == receiver {
                continue;
            }

            if resources.contains(name) {
                initialization_barriers
                    .entry(receiver)
                    .or_default()
                    .insert(sender);
            }
        }
    }

    for (name, loc) in &mut locations {
        if let Location::Owned {
            core,
            cross_initialized,
        } = loc
        {
            for (&initializer, resources) in &late_resources {
                if resources.contains(name) && *core != initializer {
                    *cross_initialized = true;
                }
            }
        }
    }

    // Most late resources need to be `Send`
    let mut send_types = SendTypes::new();
    let owned_by_idle = Ownership::Owned { priority: 0 };
    for (name, res) in app.late_resources.iter() {
        // cross-initialized || not owned by idle
        if locations
            .get(name)
            .map(|loc| loc.cross_initialized())
            .unwrap_or(false)
            || ownerships
                .get(name)
                .map(|ownership| *ownership != owned_by_idle)
                .unwrap_or(false)
        {
            if let Some(loc) = locations.get(name) {
                match loc {
                    Location::Owned { core, .. } => {
                        send_types.entry(*core).or_default().insert(res.ty.clone());
                    }

                    Location::Shared { cores } => cores.iter().for_each(|&core| {
                        send_types.entry(core).or_default().insert(res.ty.clone());
                    }),
                }
            }
        }
    }

    // All resources shared with `init` (ownership != None) need to be `Send`
    for name in app
        .inits
        .values()
        .flat_map(|init| init.args.resources.keys())
    {
        if let Some(ownership) = ownerships.get(name) {
            if *ownership != owned_by_idle {
                if let Some(loc) = locations.get(name) {
                    match loc {
                        Location::Owned { core, .. } => {
                            send_types
                                .entry(*core)
                                .or_default()
                                .insert(app.resources[name].ty.clone());
                        }

                        Location::Shared { cores } => cores.iter().for_each(|&core| {
                            send_types
                                .entry(core)
                                .or_default()
                                .insert(app.resources[name].ty.clone());
                        }),
                    }
                }
            }
        }
    }

    // Initialize the timer queues
    let mut timer_queues = TimerQueues::new();
    for (scheduler_core, _scheduler_prio, name) in app.schedule_calls() {
        let schedulee = &app.software_tasks[name];
        let schedulee_core = schedulee.args.core;
        let schedulee_prio = schedulee.args.priority;

        let tq = timer_queues.entry(scheduler_core).or_default();
        tq.tasks.insert(name.clone());

        if scheduler_core == schedulee_core {
            // the handler priority must match the priority of the highest priority schedulee that's
            // dispatched in the same core
            tq.priority = cmp::max(tq.priority, schedulee_prio);

            // the priority ceiling must be equal or greater than the handler priority
            tq.ceiling = tq.priority;
        } else {
            // when cross-scheduling the timer handler needs to run at the highest local priority
            tq.priority = app
                .hardware_tasks
                .values()
                .filter_map(|task| {
                    if task.args.core == scheduler_core {
                        Some(task.args.priority)
                    } else {
                        None
                    }
                })
                .chain(app.software_tasks.values().filter_map(|task| {
                    if task.args.core == scheduler_core {
                        Some(task.args.priority)
                    } else {
                        None
                    }
                }))
                .max()
                .map(|prio| prio + 1)
                .unwrap_or(tq.priority);

            // the priority ceiling must be equal or greater than the handler priority
            tq.ceiling = tq.priority;
        }
    }

    // g. Ceiling analysis of free queues (consumer end point) -- first pass
    // h. Ceiling analysis of the channels (producer end point) -- first pass
    // i. Spawn barriers analysis
    // j. Send analysis
    let mut channels = Channels::new();
    let mut free_queues = FreeQueues::new();
    let mut spawn_barriers = SpawnBarriers::new();
    for (spawner_core, spawner_prio, name) in app.spawn_calls() {
        let spawnee = &app.software_tasks[name];
        let spawnee_core = spawnee.args.core;
        let spawnee_prio = spawnee.args.priority;

        let mut must_be_send = false;
        if spawner_core != spawnee_core {
            // (i)
            let spawned_from_init = spawner_prio.is_none();
            spawn_barriers
                .entry(spawnee_core)
                .or_default()
                .insert(spawner_core, spawned_from_init);

            // (j) messages that cross the core boundary need to be `Send`
            must_be_send = true;
        }

        let channel = channels
            .entry(spawnee_core)
            .or_default()
            .entry(spawnee_prio)
            .or_default()
            .entry(spawner_core)
            .or_default();
        channel.tasks.insert(name.clone());

        let fq = free_queues
            .entry(name.clone())
            .or_default()
            .entry(spawner_core)
            .or_default();

        if let Some(prio) = spawner_prio {
            // (h) Spawners contend for the `channel`
            match channel.ceiling {
                None => channel.ceiling = Some(prio),
                Some(ceil) => channel.ceiling = Some(cmp::max(prio, ceil)),
            }

            // (g) Spawners contend for the free queue
            match *fq {
                None => *fq = Some(prio),
                Some(ceil) => *fq = Some(cmp::max(ceil, prio)),
            }

            // (j) core-local messages that connect tasks running at different priorities need to be
            // `Send`
            if spawner_core == spawnee_core && spawnee_prio != prio {
                must_be_send = true;
            }
        } else if spawner_core == spawnee_core {
            // (g, h) spawns from `init` are excluded from the ceiling analysis
            // (j) but spawns from `init` must be `Send`
            must_be_send = true;
        }

        if must_be_send {
            {
                let send_types = send_types.entry(spawner_core).or_default();

                spawnee.inputs.iter().for_each(|input| {
                    send_types.insert(input.ty.clone());
                });
            }

            let send_types = send_types.entry(spawnee_core).or_default();

            spawnee.inputs.iter().for_each(|input| {
                send_types.insert(input.ty.clone());
            });
        }
    }

    // k. Ceiling analysis of free queues (consumer end point) -- second pass
    // l. Ceiling analysis of the channels (producer end point) -- second pass
    // m. Ceiling analysis of the timer queue
    // n. Spawn barriers analysis (schedule edition)
    // o. Send analysis
    for (scheduler_core, scheduler_prio, name) in app.schedule_calls() {
        let schedulee = &app.software_tasks[name];
        let schedulee_core = schedulee.args.core;
        let schedulee_prio = schedulee.args.priority;

        let mut must_be_send = false;
        if scheduler_core != schedulee_core {
            // (n)
            match spawn_barriers
                .entry(schedulee_core)
                .or_default()
                .entry(scheduler_core)
            {
                // NOTE `schedule`s always send messages from the timer queue handler so they never
                // send messages during `init`
                Entry::Vacant(entry) => {
                    entry.insert(false);
                }

                Entry::Occupied(..) => {}
            }

            // (o) messages that cross the core boundary need to be `Send`
            must_be_send = true;
        }

        let tq = timer_queues.get_mut(&scheduler_core).expect("UNREACHABLE");

        let channel = channels
            .entry(schedulee_core)
            .or_default()
            .entry(schedulee_prio)
            .or_default()
            .entry(scheduler_core)
            .or_default();
        channel.tasks.insert(name.clone());

        let fq = free_queues
            .entry(name.clone())
            .or_default()
            .entry(scheduler_core)
            .or_default();

        // (l) The timer queue handler contends for the `channel`
        match channel.ceiling {
            None => channel.ceiling = Some(tq.priority),
            Some(ceil) => channel.ceiling = Some(cmp::max(ceil, tq.priority)),
        }

        if let Some(prio) = scheduler_prio {
            // (k) Schedulers contend for the free queue
            match *fq {
                None => *fq = Some(prio),
                Some(ceil) => *fq = Some(cmp::max(ceil, prio)),
            }

            // (m) Schedulers contend for the timer queue
            tq.ceiling = cmp::max(tq.ceiling, prio);

            // (o) core-local messages that connect tasks running at different priorities need to be
            // `Send`
            if scheduler_core == schedulee_core && schedulee_prio != prio {
                must_be_send = true;
            }
        } else if scheduler_core == schedulee_core {
            // (k, m) schedules from `init` are excluded from the ceiling analysis
            // (o) but schedules from `init` must be `Send`
            must_be_send = true;
        }

        if must_be_send {
            {
                let send_types = send_types.entry(scheduler_core).or_default();

                schedulee.inputs.iter().for_each(|input| {
                    send_types.insert(input.ty.clone());
                });
            }

            let send_types = send_types.entry(schedulee_core).or_default();

            schedulee.inputs.iter().for_each(|input| {
                send_types.insert(input.ty.clone());
            });
        }
    }

    // no channel should ever be empty
    debug_assert!(channels.values().all(|dispatchers| dispatchers
        .values()
        .all(|channels| channels.values().all(|channel| !channel.tasks.is_empty()))));

    // Compute channel capacities
    for channel in channels
        .values_mut()
        .flat_map(|dispatchers| dispatchers.values_mut())
        .flat_map(|dispatcher| dispatcher.values_mut())
    {
        channel.capacity = channel
            .tasks
            .iter()
            .map(|name| app.software_tasks[name].args.capacity)
            .sum();
    }

    // Compute the capacity of the timer queues
    for tq in timer_queues.values_mut() {
        tq.capacity = tq
            .tasks
            .iter()
            .map(|name| app.software_tasks[name].args.capacity)
            .sum();
    }

    let used_cores = app
        .inits
        .keys()
        .cloned()
        .chain(app.idles.keys().cloned())
        .chain(app.hardware_tasks.values().map(|task| task.args.core))
        .chain(app.software_tasks.values().map(|task| task.args.core))
        .collect();

    Analysis {
        used_cores,
        channels,
        free_queues,
        initialization_barriers,
        late_resources,
        locations,
        ownerships,
        send_types,
        spawn_barriers,
        sync_types,
        timer_queues,
    }
}

/// Priority ceiling
pub type Ceiling = Option<u8>;

/// Task priority
pub type Priority = u8;

/// Receiver core
pub type Receiver = Core;

/// Resource name
pub type Resource = Ident;

/// Sender core
pub type Sender = Core;

/// Task name
pub type Task = Ident;

/// The result of analyzing an RTFM application
pub struct Analysis {
    /// Cores that have been assigned at least task, `#[init]` or `#[idle]`
    pub used_cores: BTreeSet<Core>,

    /// SPSC message channels between cores
    pub channels: Channels,

    /// Priority ceilings of "free queues"
    pub free_queues: FreeQueues,

    /// Maps a core to the late resources it initializes
    pub late_resources: LateResources,

    /// Location of all *used* resources
    ///
    /// If a resource is not listed here it means that's a "dead" (never accessed) resource and the
    /// backend should not generate code for it
    ///
    /// `None` indicates that the resource must reside in memory visible to more than one core
    /// ("shared memory")
    pub locations: Locations,

    /// Resource ownership
    pub ownerships: Ownerships,

    /// These types must implement the `Send` trait
    pub send_types: SendTypes,

    /// These types must implement the `Sync` trait
    pub sync_types: SyncTypes,

    /// Cross-core initialization barriers
    pub initialization_barriers: InitializationBarriers,

    /// Cross-core spawn barriers
    pub spawn_barriers: SpawnBarriers,

    /// Timer queues
    pub timer_queues: TimerQueues,
}

/// All cross-core channels, keyed by receiver core, then by dispatch priority and then by sender
/// core
pub type Channels = BTreeMap<Receiver, BTreeMap<Priority, BTreeMap<Sender, Channel>>>;

/// All free queues, keyed by task and then by sender
pub type FreeQueues = IndexMap<Task, BTreeMap<Sender, Ceiling>>;

/// Late resources, keyed by the core that initializes them
pub type LateResources = BTreeMap<Core, BTreeSet<Resource>>;

/// Location of all *used* resources
pub type Locations = IndexMap<Resource, Location>;

/// Resource ownership
pub type Ownerships = IndexMap<Resource, Ownership>;

/// These types must implement the `Send` trait
pub type SendTypes = BTreeMap<Core, Set<Type>>;

/// These types must implement the `Sync` trait
pub type SyncTypes = BTreeMap<Core, Set<Type>>;

/// Cross-core initialization barriers
pub type InitializationBarriers =
    BTreeMap</* user */ Receiver, BTreeSet</* initializer */ Sender>>;

/// Cross-core spawn barriers
pub type SpawnBarriers =
    BTreeMap</* spawnee */ Receiver, BTreeMap</* spawner */ Sender, /* before_init */ bool>>;

/// Timer queues, keyed by core
pub type TimerQueues = BTreeMap<Core, TimerQueue>;

/// The timer queue
#[derive(Debug)]
pub struct TimerQueue {
    /// The capacity of the queue
    pub capacity: u8,

    /// The priority ceiling of the queue
    pub ceiling: u8,

    /// Priority of the timer queue handler
    pub priority: u8,

    /// Tasks that can be scheduled on this queue
    pub tasks: BTreeSet<Task>,
}

impl Default for TimerQueue {
    fn default() -> Self {
        Self {
            capacity: 0,
            ceiling: 1,
            priority: 1,
            tasks: BTreeSet::new(),
        }
    }
}

/// A channel between cores used to send messages
#[derive(Debug, Default)]
pub struct Channel {
    /// The channel capacity
    pub capacity: u8,

    /// The (sender side) priority ceiling of this SPSC channel
    pub ceiling: Ceiling,

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
    /// resource that resides in `core`
    Owned {
        /// Core on which this resource is located
        core: u8,

        /// Whether this resource is cross initialized
        cross_initialized: bool,
    },

    /// `Access::Shared` resource shared between different cores
    Shared {
        /// Cores that share access to this resource
        cores: BTreeSet<Core>,
    },
}

impl Location {
    /// If resource is owned this returns the core on which is located
    pub fn core(&self) -> Option<u8> {
        match *self {
            Location::Owned { core, .. } => Some(core),

            Location::Shared { .. } => None,
        }
    }

    fn cross_initialized(&self) -> bool {
        match *self {
            Location::Owned {
                cross_initialized, ..
            } => cross_initialized,
            Location::Shared { .. } => false,
        }
    }
}
