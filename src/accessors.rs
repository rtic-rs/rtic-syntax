use syn::{Expr, Ident};

use crate::{
    analyze::{Analysis, Location, Priority},
    ast::{App, HardwareTaskArgs, LateResource},
    Context, Core, Set,
};

impl App {
    /// Whether this `core` uses the `schedule` API
    pub fn uses_schedule(&self, core: u8) -> bool {
        assert!(core < self.args.cores);

        self.inits
            .get(&core)
            .map(|init| !init.args.schedule.is_empty())
            .unwrap_or(false)
            || self
                .idles
                .get(&core)
                .map(|idle| !idle.args.schedule.is_empty())
                .unwrap_or(false)
            || self
                .hardware_tasks
                .values()
                .any(|task| task.args.core == core && !task.args.schedule.is_empty())
            || self
                .software_tasks
                .values()
                .any(|task| task.args.core == core && !task.args.schedule.is_empty())
    }

    /// Returns information about the resource that matches `name`
    pub fn resource(&self, name: &Ident) -> Option<(&LateResource, Option<&Expr>)> {
        self.late_resources
            .get(name)
            .map(|late| (late, None))
            .or_else(|| {
                self.resources
                    .get(name)
                    .map(|res| (&res.late, Some(&*res.expr)))
            })
    }

    /// Returns an iterator over all *live* resources
    pub fn resources<'a>(
        &'a self,
        analysis: &'a Analysis,
    ) -> impl Iterator<Item = (&'a Ident, &'a LateResource, Option<&'a Expr>, &'a Location)> {
        analysis.locations.iter().map(move |(name, loc)| {
            let (res, expr) = self.resource(name).expect("UNREACHABLE");

            (name, res, expr, loc)
        })
    }

    /// Iterates over all spawn callers
    pub fn schedule_callers<'a>(&'a self) -> impl Iterator<Item = (Context<'a>, &'a Set<Ident>)> {
        self.inits
            .iter()
            .filter_map(|(&core, init)| {
                if !init.args.schedule.is_empty() {
                    Some((Context::Init(core), &init.args.schedule))
                } else {
                    None
                }
            })
            .chain(self.idles.iter().filter_map(|(&core, idle)| {
                if !idle.args.schedule.is_empty() {
                    Some((Context::Idle(core), &idle.args.schedule))
                } else {
                    None
                }
            }))
            .chain(self.hardware_tasks.iter().filter_map(|(name, task)| {
                if !task.args.schedule.is_empty() {
                    Some((Context::HardwareTask(name), &task.args.schedule))
                } else {
                    None
                }
            }))
            .chain(self.software_tasks.iter().filter_map(|(name, task)| {
                if !task.args.schedule.is_empty() {
                    Some((Context::SoftwareTask(name), &task.args.schedule))
                } else {
                    None
                }
            }))
    }

    /// Iterates over all spawn callers
    pub fn spawn_callers<'a>(&'a self) -> impl Iterator<Item = (Context<'a>, &'a Set<Ident>)> {
        self.inits
            .iter()
            .filter_map(|(&core, init)| {
                if !init.args.spawn.is_empty() {
                    Some((Context::Init(core), &init.args.spawn))
                } else {
                    None
                }
            })
            .chain(self.idles.iter().filter_map(|(&core, idle)| {
                if !idle.args.spawn.is_empty() {
                    Some((Context::Idle(core), &idle.args.spawn))
                } else {
                    None
                }
            }))
            .chain(self.hardware_tasks.iter().filter_map(|(name, task)| {
                if !task.args.spawn.is_empty() {
                    Some((Context::HardwareTask(name), &task.args.spawn))
                } else {
                    None
                }
            }))
            .chain(self.software_tasks.iter().filter_map(|(name, task)| {
                if !task.args.spawn.is_empty() {
                    Some((Context::SoftwareTask(name), &task.args.spawn))
                } else {
                    None
                }
            }))
    }

    pub(crate) fn resource_accesses(
        &self,
    ) -> impl Iterator<Item = (Core, Option<Priority>, &Ident)> {
        self.inits
            .iter()
            .flat_map(|(core, init)| {
                init.args
                    .resources
                    .iter()
                    .map(move |res| (*core, None, res))
            })
            .chain(self.idles.iter().flat_map(|(core, idle)| {
                idle.args
                    .resources
                    .iter()
                    .map(move |res| (*core, Some(0), res))
            }))
            .chain(self.hardware_tasks.values().flat_map(|task| {
                task.args
                    .resources
                    .iter()
                    .map(move |res| (task.args.core, Some(task.args.priority), res))
            }))
            .chain(self.software_tasks.values().flat_map(|task| {
                task.args
                    .resources
                    .iter()
                    .map(move |res| (task.args.core, Some(task.args.priority), res))
            }))
    }

    pub(crate) fn schedule_calls(&self) -> impl Iterator<Item = (Core, Option<Priority>, &Ident)> {
        self.inits
            .iter()
            .flat_map(|(&core, init)| {
                init.args
                    .schedule
                    .iter()
                    .map(move |task| (core, None, task))
            })
            .chain(self.idles.iter().flat_map(|(&core, idle)| {
                idle.args
                    .schedule
                    .iter()
                    .map(move |task| (core, Some(0), task))
            }))
            .chain(self.hardware_tasks.values().flat_map(|scheduler| {
                scheduler.args.schedule.iter().map(move |schedulee| {
                    (
                        scheduler.args.core,
                        Some(scheduler.args.priority),
                        schedulee,
                    )
                })
            }))
            .chain(self.software_tasks.values().flat_map(|scheduler| {
                scheduler.args.schedule.iter().map(move |schedulee| {
                    (
                        scheduler.args.core,
                        Some(scheduler.args.priority),
                        schedulee,
                    )
                })
            }))
    }

    /// Returns an iterator over all `spawn` calls
    ///
    /// Each spawn call includes the core spawning the task, the priority of the spawner task and
    /// the name of the spawnee. A task may appear more that once in this iterator.
    ///
    /// A priority of `None` means that this being called from `init`
    pub(crate) fn spawn_calls(&self) -> impl Iterator<Item = (Core, Option<Priority>, &Ident)> {
        self.inits
            .iter()
            .flat_map(|(&core, init)| init.args.spawn.iter().map(move |task| (core, None, task)))
            .chain(self.idles.iter().flat_map(|(&core, idle)| {
                idle.args
                    .spawn
                    .iter()
                    .map(move |task| (core, Some(0), task))
            }))
            .chain(self.hardware_tasks.values().flat_map(|spawner| {
                spawner
                    .args
                    .spawn
                    .iter()
                    .map(move |spawnee| (spawner.args.core, Some(spawner.args.priority), spawnee))
            }))
            .chain(self.software_tasks.values().flat_map(|spawner| {
                spawner
                    .args
                    .spawn
                    .iter()
                    .map(move |spawnee| (spawner.args.core, Some(spawner.args.priority), spawnee))
            }))
    }

    pub(crate) fn task_references(&self) -> impl Iterator<Item = &Ident> {
        self.inits
            .values()
            .flat_map(|init| init.args.spawn.iter().chain(&init.args.schedule))
            .chain(
                self.idles
                    .values()
                    .flat_map(|idle| idle.args.spawn.iter().chain(&idle.args.schedule)),
            )
            .chain(
                self.hardware_tasks
                    .values()
                    .flat_map(|task| task.args.spawn.iter().chain(&task.args.schedule)),
            )
            .chain(
                self.software_tasks
                    .values()
                    .flat_map(|task| task.args.spawn.iter().chain(&task.args.schedule)),
            )
    }
}

impl HardwareTaskArgs {
    /// Returns the name of the exception / interrupt this handler binds to
    pub fn binds<'a>(&'a self, handler: &'a Ident) -> &'a Ident {
        self.binds.as_ref().unwrap_or(handler)
    }
}
