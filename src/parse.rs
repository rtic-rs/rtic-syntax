
use std::collections::{HashMap, HashSet};

use syn::{Ident, Path};
use syn::synom::Synom;
use syn::Expr;
use syn::ItemStatic;
use syn::ExprArray;
use syn::LitBool;
use syn::LitInt;

use {App, Idle, Init, Resources, Statics, Task};

// TODO move this to error module ?
// Reports errors on duplicate fields in structs
macro_rules! ensure {
    ($e:expr, $field:ident, $msg:expr) => {
        if !$e {
            $field.span.unstable().error($msg).emit();
        }
    }
}

named!(parse_statics -> Statics, map!(braces!(many0!(syn!(ItemStatic))), |s| {
    let mut static_map = HashMap::new();
    let (_, statics) = s;
    for s in statics { 
        static_map.insert(s.ident, s);
    }
    static_map
}));

fn ident_from_expr_array(input: ExprArray) -> HashSet<Ident> {
    let mut idents = HashSet::new();
    for elem in input.elems {
        if let Expr::Path(expr_path) = elem {
            for ident in expr_path.path.segments {
                idents.insert(ident.ident);
            } 
        }
    }
    idents
}

named!(parse_resource_ident -> Resources, do_parse!(
    res_path_vals: alt!(syn!(ExprArray) |
                        map!(epsilon!(), |_|panic!("Failed to parse resources"))) >>
    res_set: map!(value!(res_path_vals), ident_from_expr_array) >>
    (res_set)
));

// Parse fields of a `Task`
named!(task_fields -> Task, do_parse!(
    mut task: value!(Task::default()) >>
    _loop: many0!(
        do_parse!(
            field: syn!(Ident) >>
            _colon: punct!(:) >>
            switch!(value!(field.as_ref()),
                "enabled" => map!(syn!(LitBool), |lit_bool| {
                    ensure!(task.enabled.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    let val = lit_bool.value;
                    task.enabled = Some(val);
                }) |
                "path" => map!(syn!(Path), |path| {
                    ensure!(task.path.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    task.path = Some(path);
                })|
                "priority" => map!(syn!(LitInt), |lit_int| {
                    ensure!(task.priority.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    let val = lit_int.value();
                    ensure!(val < 256, field, "priority should be less than 256");
                    task.priority = Some(val as u8);
                }) |
                "resources" => map!(parse_resource_ident, |res| {
                    ensure!(task.resources.is_none(),
                            field, 
                            format!("Duplicate field `{}`", field));
                    task.resources = Some(res);
                }) |
                _ => map!(epsilon!(), |_| {
                    field.span.unstable().error(format!("Unknown field `{}`", field)).emit();
                })
            ) >>
            // TODO ideally should parse even with an optional comma
            _comma: syn!(Token![,]) >> ()
        )
    ) >>
    (task)
));

// Parse a `Task`
named!(parse_tasks -> HashMap<Ident, Task>, do_parse!(
    mut task_map: value!(HashMap::new()) >>
    many0!(do_parse!(
        task_name: syn!(Ident) >>
        _colon: syn!(Token![:]) >>
        map!(braces!(call!(task_fields)), |(_, parsed_task)|{
            if task_map.contains_key(&task_name) {
                task_name.span.unstable().error(format!("Unknown field `{}`", task_name)).emit();
                panic!("Duplicate task {}", task_name.as_ref());
            }
            task_map.insert(task_name, parsed_task);
        }) >>
        _comma: option!(syn!(Token![,])) >> ()
    )) >>
    (task_map)
));

impl Synom for App {
    /// Parses the content of the `App` struct
    named!(parse -> App, do_parse!(
        mut app_holder: value!(App::default()) >> 
        many0!(
            do_parse!(
                field: syn!(Ident) >>
                _colon: syn!(Token![:]) >>
                switch!(value!(field.as_ref()),
                "device" => map!(syn!(Path), |path| {
                    ensure!(app_holder.device.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    app_holder.device = Some(path);
                })|
                "init" => map!(braces!(syn!(Init)), |(_, init)| {
                    ensure!(app_holder.init.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    app_holder.init = Some(init);
                }) |
                "idle" => map!(braces!(syn!(Idle)), |(_, idle)| {
                    ensure!(app_holder.idle.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    app_holder.idle = Some(idle);
                }) |
                "resources" => map!(parse_statics, |statics| {
                    ensure!(app_holder.resources.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    app_holder.resources = Some(statics);
                }) |
                "tasks" => map!(braces!(call!(parse_tasks)), |(_, tasks)| {
                    ensure!(app_holder.tasks.is_none(),
                            field,
                            format!("Duplicate field `{}`", field));
                    app_holder.tasks = Some(tasks);
                }) |
                _ => call!(|_| {
                    field.span.unstable().error(format!("Unknown field `{}`", field)).emit();
                    panic!("Unknown field `{}`", field);
                })
                ) >>
                _comma: syn!(Token![,]) >>
                (())
            )
        ) >>
        (app_holder)
    ));
}

impl Synom for Init {
    named!(parse -> Init, do_parse!(
        mut parsed_init: value!(Init::default()) >>
        many0!(
            do_parse!(
                field: syn!(Ident) >>
                _colon: punct!(:) >>
                switch!(value!(field.as_ref()), 
                    "path" => map!(syn!(Path), |path| {
                        parsed_init.path = Some(path);
                    }) |
                    "resources" => map!(parse_resource_ident, |res| {
                        parsed_init.resources = Some(res);
                    }) |
                    _ => call!(|_| {
                        field.span.unstable().error(format!("Unknown field `{}`", field)).emit();
                        panic!("Unknown field `{}`", field);
                    })
                ) >>
                _comma: syn!(Token![,]) >>
                (())
            )
        ) >>
        (parsed_init)
    ));
}

impl Synom for Idle {
    /// Parses the content of the Idle struct
    named!(parse -> Self, do_parse!(
        mut idle_parsed: value!(Idle::default()) >> 
        many0!(
            do_parse!(
                field: syn!(Ident) >>
                _colon: syn!(Token![:]) >>
                switch!(value!(field.as_ref()),
                "path" => map!(syn!(Path), |path| {
                    idle_parsed.path = Some(path);
                })|
                "resources" => map!(parse_resource_ident, |res| {
                    idle_parsed.resources = Some(res);
                }) |
                _ => call!(|_| {
                    field.span.unstable().error(format!("Unknown field `{}`", field)).emit();
                    panic!("Unknown field `{}`", field);
                })
                ) >>
                _comma: syn!(Token![,]) >>
                (())
            )
        ) >>
        (idle_parsed)
    )
    );
}
