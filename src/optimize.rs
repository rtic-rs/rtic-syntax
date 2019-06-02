use std::collections::{BTreeSet, HashMap};

use crate::{ast::App, Settings};

pub fn app(app: &mut App, settings: &Settings) {
    // "compress" priorities
    // if the user specified, for example, task priorities of "1, 3, 6"; we'll compress them into
    // "1, 2, 3" as to leave no gaps
    // this optimization is done per core -- the task priorities of one core have no effect on the
    // task scheduling of other cores
    if settings.optimize_priorities {
        for core in 0..app.args.cores {
            // all task priorities for this core ordered in ascending order
            let priorities = app
                .hardware_tasks
                .values()
                .filter_map(|task| {
                    if task.args.core == core {
                        Some(task.args.priority)
                    } else {
                        None
                    }
                })
                .chain(app.software_tasks.values().filter_map(|task| {
                    if task.args.core == core {
                        Some(task.args.priority)
                    } else {
                        None
                    }
                }))
                .collect::<BTreeSet<_>>();

            if priorities.is_empty() {
                continue;
            }

            let map = priorities
                .iter()
                .cloned()
                .zip(1..)
                .collect::<HashMap<_, _>>();

            for task in app.hardware_tasks.values_mut() {
                if task.args.core == core {
                    task.args.priority = map[&task.args.priority];
                }
            }

            for task in app.software_tasks.values_mut() {
                if task.args.core == core {
                    task.args.priority = map[&task.args.priority];
                }
            }
        }
    }
}
