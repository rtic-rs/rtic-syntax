use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use syn::parse;

use crate::ast::App;

pub fn app(app: &App) -> parse::Result<()> {
    // Check that all referenced resources have been declared
    // Check that resources are NOT `Exclusive`-ly shared
    let mut owners = HashMap::new();
    for (core, _, name, access) in app.resource_accesses() {
        if app.resource(name).is_none() {
            return Err(parse::Error::new(
                name.span(),
                "this resource has NOT been declared",
            ));
        }

        if access.is_exclusive() {
            owners.insert(name, core);
        }
    }

    // Check that no resource has both types of access (`Exclusive` & `Shared`)
    // TODO we want to allow this in the future (but behind a `Settings` feature gate)
    // accesses from `init` are not consider `Exclusive` accesses because `init` doesn't use the
    // `lock` API
    let exclusive_accesses = app
        .resource_accesses()
        .filter_map(|(_, priority, name, access)| {
            if priority.is_some() && access.is_exclusive() {
                Some(name)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();
    for (_, _, name, access) in app.resource_accesses() {
        if access.is_shared() && exclusive_accesses.contains(name) {
            return Err(parse::Error::new(
                name.span(),
                "this implementation doesn't support shared (`&-`) - exclusive (`&mut-`) locks; use `x` instead of `&x`",
            ));
        }
    }

    // Check that init only has `Access::Exclusive` resources
    // Check that late resources have NOT been assigned to `init`
    for (name, access) in app.inits.values().flat_map(|init| &init.args.resources) {
        if app.late_resources.contains_key(name) {
            return Err(parse::Error::new(
                name.span(),
                "late resources can NOT be assigned to `init`",
            ));
        }

        if access.is_shared() {
            return Err(parse::Error::new(
                name.span(),
                "`init` has direct exclusive access to resources; use `x` instead of `&x` ",
            ));
        }
    }

    // Check that all late resources are covered by `init::LateResources`
    let late_resources_set = app.late_resources.keys().collect::<HashSet<_>>();
    if late_resources_set.is_empty() {
        for init in app.inits.values() {
            if init.returns_late_resources {
                return Err(parse::Error::new(
                    init.name.span(),
                    "no late resources exist so this function must NOT return `LateResources`",
                ));
            }
        }
    } else {
        // the only core will initialize all the late resources
        if let Some(init) = app.inits.get(&0) {
            if !init.returns_late_resources {
                return Err(parse::Error::new(
                    init.name.span(),
                    "late resources exist so `#[init]` must return `init::LateResources`",
                ));
            }
        } else {
            return Err(parse::Error::new(
                Span::call_site(),
                "late resources exist so a `#[init]` function must be defined",
            ));
        }
    }

    // check that external interrupts are not used as hardware tasks
    for task in app.hardware_tasks.values() {
        let binds = &task.args.binds;

        if let Some(extern_interrupts) = app.extern_interrupts.get(&task.args.core) {
            if extern_interrupts.contains_key(binds) {
                return Err(parse::Error::new(
                    binds.span(),
                    "`extern` interrupts can't be used as hardware tasks",
                ));
            }
        }
    }

    // Check that all referenced tasks have been declared
    for task in app.task_references() {
        if app.hardware_tasks.contains_key(task) {
            return Err(parse::Error::new(
                task.span(),
                "hardware tasks can NOT be spawned",
            ));
        } else if !app.software_tasks.contains_key(task) {
            return Err(parse::Error::new(
                task.span(),
                "this software task has NOT been declared",
            ));
        }
    }

    Ok(())
}
