use syn::Ident;

use crate::{
    analyze::Priority,
    ast::{Access, App, TaskLocal},
};

impl App {
    // /// Returns information about the shared resource that matches `name`
    // pub fn resource(&self, name: &Ident) -> Option<(&SharedResource, Option<&Expr>)> {
    //     self.late_resources
    //         .get(name)
    //         .map(|late| (late, None))
    //         .or_else(|| {
    //             self.resources
    //                 .get(name)
    //                 .map(|res| (&res.late, Some(&*res.expr)))
    //         })
    // }

    // /// Returns an iterator over all *live* resources
    // pub fn resources<'a>(
    //     &'a self,
    //     analysis: &'a Analysis,
    // ) -> impl Iterator<Item = (&'a Ident, &'a LateResource, Option<&'a Expr>, &'a Location)> {
    //     analysis.locations.iter().map(move |(name, loc)| {
    //         let (res, expr) = self.resource(name).expect("UNREACHABLE");

    //         (name, res, expr, loc)
    //     })
    // }

    pub(crate) fn shared_resource_accesses(
        &self,
    ) -> impl Iterator<Item = (Option<Priority>, &Ident, Access)> {
        self.idle
            .iter()
            .flat_map(|idle| {
                idle.args
                    .shared_resources
                    .iter()
                    .map(move |(name, access)| (Some(0), name, *access))
            })
            .chain(self.hardware_tasks.values().flat_map(|task| {
                task.args
                    .shared_resources
                    .iter()
                    .map(move |(name, access)| (Some(task.args.priority), name, *access))
            }))
            .chain(self.software_tasks.values().flat_map(|task| {
                task.args
                    .shared_resources
                    .iter()
                    .map(move |(name, access)| (Some(task.args.priority), name, *access))
            }))
    }

    fn is_external(task_local: &TaskLocal) -> bool {
        match task_local {
            TaskLocal::External => true,
            _ => false,
        }
    }

    pub(crate) fn local_resource_accesses(&self) -> impl Iterator<Item = &Ident> {
        self.init
            .args
            .local_resources
            .iter()
            .map(move |(name, _)| name)
            .chain(self.idle.iter().flat_map(|idle| {
                idle.args
                    .local_resources
                    .iter()
                    .filter(|(_, task_local)| Self::is_external(task_local)) // Only check the resources declared in `#[local]`
                    .map(move |(name, _)| name)
            }))
            .chain(self.hardware_tasks.values().flat_map(|task| {
                task.args
                    .local_resources
                    .iter()
                    .filter(|(_, task_local)| Self::is_external(task_local)) // Only check the resources declared in `#[local]`
                    .map(move |(name, _)| name)
            }))
            .chain(self.software_tasks.values().flat_map(|task| {
                task.args
                    .local_resources
                    .iter()
                    .filter(|(_, task_local)| Self::is_external(task_local)) // Only check the resources declared in `#[local]`
                    .map(move |(name, _)| name)
            }))
    }
}
