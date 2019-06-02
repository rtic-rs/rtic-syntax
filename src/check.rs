use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use syn::parse;

use crate::ast::App;

pub fn app(app: &App) -> parse::Result<()> {
    let tasks_set = app
        .hardware_tasks
        .keys()
        .chain(app.software_tasks.keys())
        .collect::<HashSet<_>>();

    // Check that all referenced resources have been declared
    // Check that `static mut` resources are NOT shared between cores
    let mut owners = HashMap::new();
    for (core, _, name) in app.resource_accesses() {
        if app.resource(name).is_none() {
            return Err(parse::Error::new(
                name.span(),
                "this resource has NOT been declared",
            ));
        }

        if app
            .resource(name)
            .expect("UNREACHABLE")
            .0
            .mutability
            .is_some()
        {
            if let Some(owner) = owners.get(name) {
                if core != *owner {
                    return Err(parse::Error::new(
                        name.span(),
                        "`static mut` resources can NOT be accessed from different cores",
                    ));
                }
            } else {
                owners.insert(name, core);
            }
        }
    }

    // Check that late resources have NOT been assigned to `init`
    for res in app.inits.values().flat_map(|init| &init.args.resources) {
        if app.late_resources.contains_key(res) {
            return Err(parse::Error::new(
                res.span(),
                "late resources can NOT be assigned to `init`",
            ));
        }
    }

    // Check that all late resources are covered by `init::LateResources`
    let cores = app.args.cores;
    let mut late_resources_set = app.late_resources.keys().collect::<HashSet<_>>();
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
        if cores == 1 {
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
        } else {
            // this core will initialize the "rest" of late resources
            let mut rest = None;

            let mut initialized = HashMap::new();
            for (core, init) in &app.inits {
                if !init.returns_late_resources {
                    continue;
                }

                if late_resources_set.is_empty() {
                    return Err(parse::Error::new(
                        init.name.span(),
                        "no more late resources to initialize; \
                         this function must NOT return `LateResources`",
                    ));
                }

                if !init.args.late.is_empty() {
                    for res in &init.args.late {
                        if !app.late_resources.contains_key(res) {
                            return Err(parse::Error::new(
                                res.span(),
                                "this is not a late resource",
                            ));
                        }

                        if let Some(other) = initialized.get(res) {
                            return Err(parse::Error::new(
                                res.span(),
                                &format!("this resource is initialized by core {}", other),
                            ));
                        } else {
                            late_resources_set.remove(res);
                            initialized.insert(res, core);
                        }
                    }
                } else if let Some(rest) = rest {
                    return Err(parse::Error::new(
                        init.name.span(),
                        &format!(
                            "unclear how initialization of late resources is split between \
                             cores {} and {}",
                            rest, core,
                        ),
                    ));
                } else {
                    rest = Some(core);
                }
            }

            if let Some(res) = late_resources_set.iter().next() {
                if rest.is_none() {
                    return Err(parse::Error::new(
                        res.span(),
                        "this resource is not being initialized",
                    ));
                }
            }
        }
    }

    // Check that all referenced tasks have been declared
    for task in app.task_references() {
        if !tasks_set.contains(task) {
            return Err(parse::Error::new(
                task.span(),
                "this task has NOT been declared",
            ));
        }
    }

    Ok(())
}
