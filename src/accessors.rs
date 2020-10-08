use syn::{Expr, Ident};

use crate::{
    analyze::{Analysis, Location, Priority},
    ast::{Access, App, LateResource},
};

impl App {
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
                task.args
                    .resources
                    .iter()
                    .map(move |(name, access)| (Some(task.args.priority), name, *access))
            }))
            .chain(self.software_tasks.values().flat_map(|task| {
                task.args
                    .resources
                    .iter()
                    .map(move |(name, access)| (Some(task.args.priority), name, *access))
            }))
    }
}
