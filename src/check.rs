use std::collections::HashMap;

use quote::Tokens;
use syn::Ident;

use error::*;
use {Idents, Statics};

pub type Tasks = HashMap<Ident, Task>;

pub struct App {
    pub device: Tokens,
    pub idle: Idle,
    pub init: Init,
    pub resources: Statics,
    pub tasks: Tasks,
}

pub struct Idle {
    pub locals: Statics,
    pub path: Tokens,
    pub resources: Idents,
}

pub struct Init {
    pub path: Tokens,
}

pub struct Task {
    pub enabled: Option<bool>,
    pub priority: Option<u8>,
    pub resources: Idents,
}

pub fn app(app: ::App) -> Result<App> {
    Ok(App {
        device: app.device,
        idle: ::check::idle(app.idle).chain_err(|| "checking `idle`")?,
        init: ::check::init(app.init).chain_err(|| "checking `init`")?,
        resources: ::check::statics("resources", app.resources)
            .chain_err(|| "checking `resources`")?,
        tasks: ::check::tasks(app.tasks).chain_err(|| "checking `tasks`")?,
    })
}

fn idents(field: &str, idents: Option<Idents>) -> Result<Idents> {
    Ok(if let Some(idents) = idents {
        ensure!(
            !idents.is_empty(),
            "empty `{}` field. It should be removed.",
            field
        );

        idents
    } else {
        Idents::new()
    })
}

fn idle(idle: Option<::Idle>) -> Result<Idle> {
    Ok(if let Some(idle) = idle {
        ensure!(
            idle.locals.is_some() || idle.path.is_some() ||
                idle.resources.is_some(),
            "empty `idle` field. It should be removed."
        );

        Idle {
            locals: ::check::statics("locals", idle.locals)?,
            path: ::check::path("idle", idle.path)
                .chain_err(|| "checking `path`")?,
            resources: ::check::idents("resources", idle.resources)?,
        }
    } else {
        Idle {
            locals: Statics::new(),
            path: quote!(idle),
            resources: Idents::new(),
        }
    })
}

fn init(init: Option<::Init>) -> Result<Init> {
    Ok(if let Some(init) = init {
        if let Some(path) = init.path {
            Init {
                path: ::check::path("init", Some(path))
                    .chain_err(|| "checking `path`")?,
            }
        } else {
            bail!("empty `init` field. It should be removed.");
        }
    } else {
        Init { path: quote!(init) }
    })
}

fn path(default: &str, path: Option<Tokens>) -> Result<Tokens> {
    Ok(if let Some(path) = path {
        ensure!(
            path.as_str() != default,
            "this is the default value. It should be omitted."
        );

        path
    } else {
        let default = Ident::new(default);
        quote!(#default)
    })
}

fn statics(field: &str, statics: Option<Statics>) -> Result<Statics> {
    Ok(if let Some(statics) = statics {
        ensure!(
            !statics.is_empty(),
            "empty `{}` field. It should be removed.",
            field
        );

        statics
    } else {
        Statics::new()
    })
}

fn tasks(tasks: Option<::Tasks>) -> Result<Tasks> {
    Ok(if let Some(tasks) = tasks {
        ensure!(
            !tasks.is_empty(),
            "empty `tasks` field. It should be removed"
        );

        tasks
            .into_iter()
            .map(|(name, task)| {
                Ok((
                    name.clone(),
                    Task {
                        enabled: task.enabled,
                        priority: task.priority,
                        resources: ::check::idents("resources", task.resources)
                            .chain_err(|| format!("checking task `{}`", name))?,
                    },
                ))
            })
            .collect::<Result<_>>()?
    } else {
        Tasks::new()
    })
}
