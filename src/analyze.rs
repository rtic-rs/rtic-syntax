//! RTIC application analysis

use core::cmp;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use indexmap::IndexMap;
use quote::format_ident;
use syn::{Ident, Type};

use crate::{ast::App, Set};

pub(crate) fn app(app: &App) -> Result<Analysis, syn::Error> {
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

    // Collect task local resources
    let task_local: Vec<&Ident> = app
        .resources
        .iter()
        .filter(|(_, r)| r.properties.task_local)
        .map(|(i, _)| i)
        .chain(
            app.late_resources
                .iter()
                .filter(|(_, r)| r.properties.task_local)
                .map(|(i, _)| i),
        )
        .collect();

    // Collect lock free resources
    let lock_free: Vec<&Ident> = app
        .resources
        .iter()
        .filter(|(_, r)| r.properties.lock_free)
        .map(|(i, _)| i)
        .chain(
            app.late_resources
                .iter()
                .filter(|(_, r)| r.properties.lock_free)
                .map(|(i, _)| i),
        )
        .collect();

    // Collect all tasks into a vector
    type TaskName = String;
    type Priority = u8;

    // The task list is a Tuple (Name, Resources, Priority)
    let task_list: Vec<(TaskName, Vec<&Ident>, Priority)> = app
        .idles
        .iter()
        .map(|ht| {
            (
                "idle".to_string(),
                ht.args.resources.iter().map(|(v, _)| v).collect::<Vec<_>>(),
                0,
            )
        })
        .chain(app.software_tasks.iter().map(|(name, ht)| {
            (
                name.to_string(),
                ht.args.resources.iter().map(|(v, _)| v).collect::<Vec<_>>(),
                ht.args.priority,
            )
        }))
        .chain(app.hardware_tasks.iter().map(|(name, ht)| {
            (
                name.to_string(),
                ht.args.resources.iter().map(|(v, _)| v).collect::<Vec<_>>(),
                ht.args.priority,
            )
        }))
        .collect();

    // Create the list of task Idents
    let tasks = task_list.iter().map(|x| format_ident!("{}", x.0)).collect();

    // Check that task_local resources are only used once
    let mut error = vec![];
    for task_local_id in task_local.iter() {
        let mut used = vec![];
        for (task, tr, priority) in task_list.iter() {
            for r in tr {
                if task_local_id == r {
                    used.push((task, r, priority));
                }
            }
        }
        if used.len() > 1 {
            error.push(syn::Error::new(
                task_local_id.span(),
                format!(
                    "task local resource {:?} is used by multiple tasks",
                    task_local_id.to_string()
                ),
            ));

            used.iter().for_each(|(task, resource, priority)| {
                error.push(syn::Error::new(
                    resource.span(),
                    format!(
                        "task local resource {:?} is used by task {:?} with priority {:?}",
                        resource.to_string(),
                        task,
                        priority
                    ),
                ))
            });
        }
    }

    let mut lf_res_with_error = vec![];
    let mut lf_hash = HashMap::new();

    // Check that lock_free resources are correct
    for lf_res in lock_free.iter() {
        for (task, tr, priority) in task_list.iter() {
            for r in tr {
                // Get all uses of resources annotated lock_free
                if lf_res == r {
                    // HashMap returns the previous existing object if old.key == new.key
                    if let Some(lf_res) = lf_hash.insert(r.to_string(), (task, r, priority)) {
                        // Check if priority differ, if it does, append to
                        // list of resources which will be annotated with errors
                        if priority != lf_res.2 {
                            lf_res_with_error.push(lf_res.1);
                            lf_res_with_error.push(r);
                        }
                        // If the resource already violates lock free properties
                        if lf_res_with_error.contains(&r) {
                            lf_res_with_error.push(lf_res.1);
                            lf_res_with_error.push(r);
                        }
                    }
                }
            }
        }
    }

    // Add error message in the resource struct
    for r in lock_free {
        if lf_res_with_error.contains(&&r) {
            error.push(syn::Error::new(
                r.span(),
                format!(
                    "Lock free resource {:?} is used by tasks at different priorities",
                    r.to_string(),
                ),
            ));
        }
    }

    // Add error message for each use of the resource
    for resource in lf_res_with_error.clone() {
        error.push(syn::Error::new(
            resource.span(),
            format!(
                "Resource {:?} is declared lock free but used by tasks at different priorities",
                resource.to_string(),
            ),
        ));
    }

    // Collect errors if any and return/halt
    if !error.is_empty() {
        let mut err = error.iter().next().unwrap().clone();
        error.iter().for_each(|e| err.combine(e.clone()));
        return Err(err);
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

    // Initialize the timer queues
    let mut timer_queues = TimerQueues::new();
    for (_, name) in app.schedule_calls() {
        let schedulee = &app.software_tasks[name];
        let schedulee_prio = schedulee.args.priority;

        // Get the TimerQueue
        // If there is no TimerQueue, create one
        if timer_queues.is_empty() {
            timer_queues.push(TimerQueue::default());
        }
        let mut tq = timer_queues.first_mut().unwrap();
        tq.tasks.insert(name.clone());

        // the handler priority must match the priority of the highest priority schedulee
        tq.priority = cmp::max(tq.priority, schedulee_prio);

        // the priority ceiling must be equal or greater than the handler priority
        tq.ceiling = tq.priority;
    }

    // g. Ceiling analysis of free queues (consumer end point) -- first pass
    // h. Ceiling analysis of the channels (producer end point) -- first pass
    // j. Send analysis
    let mut channels = Channels::new();
    let mut free_queues = FreeQueues::new();
    for (spawner_prio, name) in app.spawn_calls() {
        let spawnee = &app.software_tasks[name];
        let spawnee_prio = spawnee.args.priority;

        let mut must_be_send = false;

        let channel = channels.entry(spawnee_prio).or_default();
        channel.tasks.insert(name.clone());

        let fq = free_queues.entry(name.clone()).or_default();

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

            // (j) messages that connect tasks running at different priorities need to be
            // `Send`
            if spawnee_prio != prio {
                must_be_send = true;
            }
        } else {
            // (g, h) spawns from `init` are excluded from the ceiling analysis
            // (j) but spawns from `init` must be `Send`
            must_be_send = true;
        }

        if must_be_send {
            {
                spawnee.inputs.iter().for_each(|input| {
                    send_types.insert(input.ty.clone());
                });
            }

            spawnee.inputs.iter().for_each(|input| {
                send_types.insert(input.ty.clone());
            });
        }
    }

    // k. Ceiling analysis of free queues (consumer end point) -- second pass
    // l. Ceiling analysis of the channels (producer end point) -- second pass
    // m. Ceiling analysis of the timer queue
    // o. Send analysis

    for (scheduler_prio, name) in app.schedule_calls() {
        let schedulee = &app.software_tasks[name];
        let schedulee_prio = schedulee.args.priority;

        let mut must_be_send = false;

        let channel = channels.entry(schedulee_prio).or_default();
        channel.tasks.insert(name.clone());

        // Get the TimerQueue
        // If there is no TimerQueue, create one
        if timer_queues.is_empty() {
            timer_queues.push(TimerQueue::default());
        }
        let mut tq = timer_queues.first_mut().unwrap();

        let fq = free_queues.entry(name.clone()).or_default();

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

            // (o) messages that connect tasks running at different priorities need to be
            // `Send`
            if schedulee_prio != prio {
                must_be_send = true;
            }
        } else {
            // (k, m) schedules from `init` are excluded from the ceiling analysis
            // (o) but schedules from `init` must be `Send`
            must_be_send = true;
        }

        if must_be_send {
            {
                schedulee.inputs.iter().for_each(|input| {
                    send_types.insert(input.ty.clone());
                });
            }

            schedulee.inputs.iter().for_each(|input| {
                send_types.insert(input.ty.clone());
            });
        }
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

    // Compute the capacity of the timer queues
    if let Some(tq) = timer_queues.first_mut() {
        tq.capacity = tq
            .tasks
            .iter()
            .map(|name| app.software_tasks[name].args.capacity)
            .sum();
    }

    Ok(Analysis {
        channels,
        free_queues,
        late_resources,
        locations,
        tasks,
        ownerships,
        send_types,
        sync_types,
        timer_queues,
    })
}

/// Priority ceiling
pub type Ceiling = Option<u8>;

/// Task priority
pub type Priority = u8;

/// Resource name
pub type Resource = Ident;

/// Task name
pub type Task = Ident;

/// List of tasks names
pub type Tasks = Vec<Ident>;

/// The result of analyzing an RTIC application
pub struct Analysis {
    /// SPSC message channels
    pub channels: Channels,

    /// Priority ceilings of "free queues"
    pub free_queues: FreeQueues,

    /// The late resources
    pub late_resources: LateResources,

    /// Location of all *used* resources
    ///
    /// If a resource is not listed here it means that's a "dead" (never accessed) resource and the
    /// backend should not generate code for it
    ///
    /// `None` indicates that the resource must reside in shared memory
    pub locations: Locations,

    /// A vector containing all task names
    pub tasks: Tasks,

    /// Resource ownership
    pub ownerships: Ownerships,

    /// These types must implement the `Send` trait
    pub send_types: SendTypes,

    /// These types must implement the `Sync` trait
    pub sync_types: SyncTypes,

    /// Timer queues
    pub timer_queues: TimerQueues,
}

/// All channels, keyed by dispatch priority
pub type Channels = BTreeMap<Priority, Channel>;

/// All free queues, keyed by task and containing the Ceiling
pub type FreeQueues = IndexMap<Task, Ceiling>;

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

/// Timer queue, wrapped in a vec!
pub type TimerQueues = Vec<TimerQueue>;

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

/// A channel used to send messages
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
    /// resource that is owned
    Owned,
}
