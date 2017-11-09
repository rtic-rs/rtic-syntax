//! Syntax checking pass

use std::collections::HashMap;

use syn::{Ident, Path};

use error::*;
use {util, Resources, Statics};

/// `$($Ident: { .. },)*`
pub type Tasks = HashMap<Ident, Task>;

/// `app! { .. }`
#[derive(Debug)]
pub struct App {
    /// `device: $path`
    pub device: Path,
    /// `idle: { $Idle }`
    pub idle: Idle,
    /// `init: { $Init }`
    pub init: Init,
    /// `resources: $Statics`
    pub resources: Statics,
    /// `root: $path`
    pub root: Option<Path>,
    /// `tasks: { $Tasks }`
    pub tasks: Tasks,
    _extensible: (),
}

/// `idle: { .. }`
#[derive(Debug)]
pub struct Idle {
    /// `path: $Path`
    pub path: Path,
    /// `resources: $Resources`
    pub resources: Resources,
    _extensible: (),
}

/// `init: { .. }`
#[derive(Debug)]
pub struct Init {
    /// `path: $Path`
    pub path: Path,
    _extensible: (),
}

/// `$Ident: { .. }`
#[derive(Debug)]
pub struct Task {
    /// `enabled: $bool`
    pub enabled: Option<bool>,
    /// `path: $Path`
    pub path: Option<Path>,
    /// `priority: $u8`
    pub priority: Option<u8>,
    /// `resources: $Resources`
    pub resources: Resources,
    _extensible: (),
}

/// Checks the syntax of the parsed `app!` macro
pub fn app(app: ::App) -> Result<App> {
    Ok(App {
        _extensible: (),
        device: app.device,
        idle: ::check::idle(app.idle).chain_err(|| "checking `idle`")?,
        init: ::check::init(app.init).chain_err(|| "checking `init`")?,
        resources: ::check::statics("resources", app.resources)
            .chain_err(|| "checking `resources`")?,
        root: app.root,
        tasks: ::check::tasks(app.tasks).chain_err(|| "checking `tasks`")?,
    })
}

fn idle(idle: Option<::Idle>) -> Result<Idle> {
    Ok(if let Some(idle) = idle {
        ensure!(
            idle.path.is_some() || idle.resources.is_some(),
            "empty `idle` field. It should be removed."
        );

        Idle {
            _extensible: (),
            path: ::check::path("idle", idle.path)
                .chain_err(|| "checking `path`")?,
            resources: ::check::resources("resources", idle.resources)?,
        }
    } else {
        Idle {
            _extensible: (),
            path: util::mk_path("idle"),
            resources: Resources::new(),
        }
    })
}

fn init(init: Option<::Init>) -> Result<Init> {
    Ok(if let Some(init) = init {
        if let Some(path) = init.path {
            Init {
                _extensible: (),
                path: ::check::path("init", Some(path))
                    .chain_err(|| "checking `path`")?,
            }
        } else {
            bail!("empty `init` field. It should be removed.");
        }
    } else {
        Init {
            _extensible: (),
            path: util::mk_path("init"),
        }
    })
}

fn path(default: &str, path: Option<Path>) -> Result<Path> {
    Ok(if let Some(path) = path {
        ensure!(
            path.segments.len() != 1 ||
                path.segments[0].ident.as_ref() != default,
            "this is the default value. It should be omitted."
        );

        path
    } else {
        util::mk_path(default)
    })
}

fn resources(field: &str, idents: Option<Resources>) -> Result<Resources> {
    Ok(if let Some(idents) = idents {
        ensure!(
            !idents.is_empty(),
            "empty `{}` field. It should be removed.",
            field
        );

        idents
    } else {
        Resources::new()
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
            .map(|(name_, task)| {
                let name = name_.clone();
                (move || -> Result<_> {
                    Ok((
                        name,
                        Task {
                            _extensible: (),
                            enabled: task.enabled,
                            path: task.path,
                            priority: task.priority,
                            resources: ::check::resources(
                                "resources",
                                task.resources,
                            )?,
                        },
                    ))
                })().chain_err(|| format!("checking task `{}`", name_))
            })
            .collect::<Result<_>>()?
    } else {
        Tasks::new()
    })
}
