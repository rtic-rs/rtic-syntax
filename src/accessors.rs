use syn::{Expr, Ident};

use crate::{
    analyze::{Analysis, Location, Priority},
    ast::{Access, App, LateResource},
    Context, Set,
};

impl App {
    /// Whether the `schedule` API is used
    pub fn uses_schedule(&self) -> bool {
        self.inits
            .first()
            .map(|init| !init.args.schedule.is_empty())
            .unwrap_or(false)
            || self
                .idles
                .first()
                .map(|idle| !idle.args.schedule.is_empty())
                .unwrap_or(false)
            || self
                .hardware_tasks
                .values()
                .any(|task| !task.args.schedule.is_empty())
            || self
                .software_tasks
                .values()
                .any(|task| !task.args.schedule.is_empty())
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
            .filter_map(|init| {
                if !init.args.schedule.is_empty() {
                    Some((Context::Init, &init.args.schedule))
                } else {
                    None
                }
            })
            .chain(self.idles.iter().filter_map(|idle| {
                if !idle.args.schedule.is_empty() {
                    Some((Context::Idle, &idle.args.schedule))
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
            .filter_map(|init| {
                if !init.args.spawn.is_empty() {
                    Some((Context::Init, &init.args.spawn))
                } else {
                    None
                }
            })
            .chain(self.idles.iter().filter_map(|idle| {
                if !idle.args.spawn.is_empty() {
                    Some((Context::Idle, &idle.args.spawn))
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
    ) -> impl Iterator<Item = (Option<Priority>, &Ident, Access)> {
        self.inits
            .iter()
            .flat_map(|init| {
                init.args
                    .resources
                    .iter()
                    .map(move |(name, access)| (None, name, *access))
            })
            .chain(self.idles.iter().flat_map(|idle| {
                idle.args
                    .resources
                    .iter()
                    .map(move |(name, access)| (Some(0), name, *access))
            }))
            .chain(self.hardware_tasks.values().flat_map(|task| {
                task.args.resources.iter().map(move |(name, access)| {
                    (Some(task.args.priority), name, *access)
                })
            }))
            .chain(self.software_tasks.values().flat_map(|task| {
                task.args.resources.iter().map(move |(name, access)| {
                    (Some(task.args.priority), name, *access)
                })
            }))
    }

    pub(crate) fn schedule_calls(&self) -> impl Iterator<Item = (Option<Priority>, &Ident)> {
        self.inits
            .iter()
            .flat_map(|init| {
                init.args
                    .schedule
                    .iter()
                    .map(move |task| (None, task))
            })
            .chain(self.idles.iter().flat_map(|idle| {
                idle.args
                    .schedule
                    .iter()
                    .map(move |task| (Some(0), task))
            }))
            .chain(self.hardware_tasks.values().flat_map(|scheduler| {
                scheduler.args.schedule.iter().map(move |schedulee| {
                    (
                        Some(scheduler.args.priority),
                        schedulee,
                    )
                })
            }))
            .chain(self.software_tasks.values().flat_map(|scheduler| {
                scheduler.args.schedule.iter().map(move |schedulee| {
                    (
                        Some(scheduler.args.priority),
                        schedulee,
                    )
                })
            }))
    }

    /// Returns an iterator over all `spawn` calls
    ///
    /// Each spawn call includes the priority of the spawner task and
    /// the name of the spawnee. A task may appear more that once in this iterator.
    ///
    /// A priority of `None` means that this being called from `init`
    pub(crate) fn spawn_calls(&self) -> impl Iterator<Item = (Option<Priority>, &Ident)> {
        self.inits
            .iter()
            .flat_map(|init| init.args.spawn.iter().map(move |task| (None, task)))
            .chain(self.idles.iter().flat_map(|idle| {
                idle.args
                    .spawn
                    .iter()
                    .map(move |task| (Some(0), task))
            }))
            .chain(self.hardware_tasks.values().flat_map(|spawner| {
                spawner
                    .args
                    .spawn
                    .iter()
                    .map(move |spawnee| (Some(spawner.args.priority), spawnee))
            }))
            .chain(self.software_tasks.values().flat_map(|spawner| {
                spawner
                    .args
                    .spawn
                    .iter()
                    .map(move |spawnee| (Some(spawner.args.priority), spawnee))
            }))
    }

    pub(crate) fn task_references(&self) -> impl Iterator<Item = &Ident> {
        self.inits
            .iter()
            .flat_map(|init| init.args.spawn.iter().chain(&init.args.schedule))
            .chain(
                self.idles
                    .iter()
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
