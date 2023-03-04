use std::collections::HashSet;

// use indexmap::map::Entry;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{self, ParseStream, Parser},
    spanned::Spanned,
    Expr, ExprArray, Fields, ForeignItem, Ident, Item, LitBool, Path, Token, Type, Visibility,
};

use super::Input;
use crate::{
    ast::{
        Actor, App, AppArgs, ExternInterrupt, ExternInterrupts, HardwareTask, Idle, IdleArgs, Init,
        InitArgs, LocalResource, Monotonic, MonotonicArgs, SharedResource, SoftwareTask,
    },
    parse::util,
    Either, Map, Set, Settings,
};

impl AppArgs {
    pub(crate) fn parse(tokens: TokenStream2) -> parse::Result<Self> {
        (|input: ParseStream<'_>| -> parse::Result<Self> {
            let mut custom = Set::new();
            let mut device = None;
            let mut peripherals = true;
            let mut extern_interrupts = ExternInterrupts::new();

            loop {
                if input.is_empty() {
                    break;
                }

                // #ident = ..
                let ident: Ident = input.parse()?;
                let _eq_token: Token![=] = input.parse()?;

                if custom.contains(&ident) {
                    return Err(parse::Error::new(
                        ident.span(),
                        "argument appears more than once",
                    ));
                }

                custom.insert(ident.clone());

                let ks = ident.to_string();

                match &*ks {
                    "device" => {
                        if let Ok(p) = input.parse::<Path>() {
                            device = Some(p);
                        } else {
                            return Err(parse::Error::new(
                                ident.span(),
                                "unexpected argument value; this should be a path",
                            ));
                        }
                    }

                    "peripherals" => {
                        if let Ok(p) = input.parse::<LitBool>() {
                            peripherals = p.value;
                        } else {
                            return Err(parse::Error::new(
                                ident.span(),
                                "unexpected argument value; this should be a boolean",
                            ));
                        }
                    }

                    "dispatchers" => {
                        if let Ok(p) = input.parse::<ExprArray>() {
                            for e in p.elems {
                                match e {
                                    Expr::Path(ep) => {
                                        let path = ep.path;
                                        let ident = if path.leading_colon.is_some()
                                            || path.segments.len() != 1
                                        {
                                            return Err(parse::Error::new(
                                                path.span(),
                                                "interrupt must be an identifier, not a path",
                                            ));
                                        } else {
                                            path.segments[0].ident.clone()
                                        };
                                        let span = ident.span();
                                        if extern_interrupts.contains_key(&ident) {
                                            return Err(parse::Error::new(
                                                span,
                                                "this extern interrupt is listed more than once",
                                            ));
                                        } else {
                                            extern_interrupts
                                                .insert(ident, ExternInterrupt { attrs: ep.attrs });
                                        }
                                    }
                                    _ => {
                                        return Err(parse::Error::new(
                                            e.span(),
                                            "interrupt must be an identifier",
                                        ));
                                    }
                                }
                            }
                        } else {
                            return Err(parse::Error::new(
                                ident.span(),
                                // increasing the length of the error message will break rustfmt
                                "unexpected argument value; expected an array",
                            ));
                        }
                    }
                    _ => {
                        return Err(parse::Error::new(ident.span(), "unexpected argument"));
                    }
                }

                if input.is_empty() {
                    break;
                }

                // ,
                let _: Token![,] = input.parse()?;
            }

            Ok(AppArgs {
                device,
                peripherals,
                extern_interrupts,
            })
        })
        .parse2(tokens)
    }
}

impl App {
    pub(crate) fn parse(args: AppArgs, input: Input, settings: &Settings) -> parse::Result<Self> {
        let mut init = None;
        let mut idle = None;

        let mut shared_resources_ident = None;
        let mut shared_resources = Map::new();
        let mut local_resources_ident = None;
        let mut actors_ident = None;
        let mut local_resources = Map::new();
        let mut actors = Map::new();
        let mut monotonics = Map::new();
        let mut hardware_tasks = Map::new();
        let mut software_tasks = Map::new();
        let mut user_imports = vec![];
        let mut user_code = vec![];

        let mut seen_idents = HashSet::<Ident>::new();
        let mut bindings = HashSet::<Ident>::new();
        let mut monotonic_types = HashSet::<Type>::new();

        let mut check_binding = |ident: &Ident| {
            if bindings.contains(ident) {
                return Err(parse::Error::new(
                    ident.span(),
                    "this interrupt is already bound",
                ));
            } else {
                bindings.insert(ident.clone());
            }

            Ok(())
        };

        let mut check_ident = |ident: &Ident| {
            if seen_idents.contains(ident) {
                return Err(parse::Error::new(
                    ident.span(),
                    "this identifier has already been used",
                ));
            } else {
                seen_idents.insert(ident.clone());
            }

            Ok(())
        };

        let mut check_monotonic = |ty: &Type| {
            if monotonic_types.contains(ty) {
                return Err(parse::Error::new(
                    ty.span(),
                    "this type is already used by another monotonic",
                ));
            } else {
                monotonic_types.insert(ty.clone());
            }

            Ok(())
        };

        for mut item in input.items {
            match item {
                Item::Fn(mut item) => {
                    let span = item.sig.ident.span();
                    if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "init"))
                    {
                        let args = InitArgs::parse(item.attrs.remove(pos).tokens)?;

                        // If an init function already exists, error
                        if init.is_some() {
                            return Err(parse::Error::new(
                                span,
                                "`#[init]` function must appear at most once",
                            ));
                        }

                        check_ident(&item.sig.ident)?;

                        init = Some(Init::parse(args, item)?);
                    } else if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "idle"))
                    {
                        let args = IdleArgs::parse(item.attrs.remove(pos).tokens)?;

                        // If an idle function already exists, error
                        if idle.is_some() {
                            return Err(parse::Error::new(
                                span,
                                "`#[idle]` function must appear at most once",
                            ));
                        }

                        check_ident(&item.sig.ident)?;

                        idle = Some(Idle::parse(args, item)?);
                    } else if let Some(pos) = item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "task"))
                    {
                        if hardware_tasks.contains_key(&item.sig.ident)
                            || software_tasks.contains_key(&item.sig.ident)
                        {
                            return Err(parse::Error::new(
                                span,
                                "this task is defined multiple times",
                            ));
                        }

                        match crate::parse::task_args(item.attrs.remove(pos).tokens, settings)? {
                            Either::Left(args) => {
                                check_binding(&args.binds)?;
                                check_ident(&item.sig.ident)?;

                                hardware_tasks.insert(
                                    item.sig.ident.clone(),
                                    HardwareTask::parse(args, item)?,
                                );
                            }

                            Either::Right(args) => {
                                check_ident(&item.sig.ident)?;

                                software_tasks.insert(
                                    item.sig.ident.clone(),
                                    SoftwareTask::parse(args, item)?,
                                );
                            }
                        }
                    } else {
                        // Forward normal functions
                        user_code.push(Item::Fn(item.clone()));
                    }
                }

                Item::Struct(ref mut struct_item) => {
                    // Match structures with the attribute #[shared], name of structure is not
                    // important
                    if let Some(_pos) = struct_item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "shared"))
                    {
                        let span = struct_item.ident.span();

                        shared_resources_ident = Some(struct_item.ident.clone());

                        if !shared_resources.is_empty() {
                            return Err(parse::Error::new(
                                span,
                                "`#[shared]` struct must appear at most once",
                            ));
                        }

                        if struct_item.vis != Visibility::Inherited {
                            return Err(parse::Error::new(
                                struct_item.span(),
                                "this item must have inherited / private visibility",
                            ));
                        }

                        if let Fields::Named(fields) = &mut struct_item.fields {
                            for field in &mut fields.named {
                                let ident = field.ident.as_ref().expect("UNREACHABLE");

                                if shared_resources.contains_key(ident) {
                                    return Err(parse::Error::new(
                                        ident.span(),
                                        "this resource is listed more than once",
                                    ));
                                }

                                shared_resources.insert(
                                    ident.clone(),
                                    SharedResource::parse(field, ident.span())?,
                                );
                            }
                        } else {
                            return Err(parse::Error::new(
                                struct_item.span(),
                                "this `struct` must have named fields",
                            ));
                        }
                    } else if let Some(_pos) = struct_item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "local"))
                    {
                        let span = struct_item.ident.span();

                        local_resources_ident = Some(struct_item.ident.clone());

                        if !local_resources.is_empty() {
                            return Err(parse::Error::new(
                                span,
                                "`#[local]` struct must appear at most once",
                            ));
                        }

                        if struct_item.vis != Visibility::Inherited {
                            return Err(parse::Error::new(
                                struct_item.span(),
                                "this item must have inherited / private visibility",
                            ));
                        }

                        if let Fields::Named(fields) = &mut struct_item.fields {
                            for field in &mut fields.named {
                                let ident = field.ident.as_ref().expect("UNREACHABLE");

                                if local_resources.contains_key(ident) {
                                    return Err(parse::Error::new(
                                        ident.span(),
                                        "this resource is listed more than once",
                                    ));
                                }

                                local_resources.insert(
                                    ident.clone(),
                                    LocalResource::parse(field, ident.span())?,
                                );
                            }
                        } else {
                            return Err(parse::Error::new(
                                struct_item.span(),
                                "this `struct` must have named fields",
                            ));
                        }
                    } else if let Some(_pos) = struct_item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "actors"))
                    {
                        let span = struct_item.ident.span();

                        actors_ident = Some(struct_item.ident.clone());

                        if !actors.is_empty() {
                            return Err(parse::Error::new(
                                span,
                                "`#[actors]` struct must appear at most once",
                            ));
                        }

                        if struct_item.vis != Visibility::Inherited {
                            return Err(parse::Error::new(
                                struct_item.span(),
                                "this item must have inherited / private visibility",
                            ));
                        }

                        if let Fields::Named(fields) = &mut struct_item.fields {
                            for field in &mut fields.named {
                                let ident = field.ident.as_ref().expect("UNREACHABLE");

                                if actors.contains_key(ident) {
                                    return Err(parse::Error::new(
                                        ident.span(),
                                        "this resource is listed more than once",
                                    ));
                                }

                                actors.insert(ident.clone(), Actor::parse(field, ident.span())?);
                            }
                        } else {
                            return Err(parse::Error::new(
                                struct_item.span(),
                                "this `struct` must have named fields",
                            ));
                        }
                    } else {
                        // Structure without the #[resources] attribute should just be passed along
                        user_code.push(item.clone());
                    }
                }

                Item::ForeignMod(mod_) => {
                    if !util::abi_is_rust(&mod_.abi) {
                        return Err(parse::Error::new(
                            mod_.abi.extern_token.span(),
                            "this `extern` block must use the \"Rust\" ABI",
                        ));
                    }

                    for item in mod_.items {
                        if let ForeignItem::Fn(mut item) = item {
                            let span = item.sig.ident.span();
                            if let Some(pos) = item
                                .attrs
                                .iter()
                                .position(|attr| util::attr_eq(attr, "task"))
                            {
                                if hardware_tasks.contains_key(&item.sig.ident)
                                    || software_tasks.contains_key(&item.sig.ident)
                                {
                                    return Err(parse::Error::new(
                                        span,
                                        "this task is defined multiple times",
                                    ));
                                }

                                if item.attrs.len() != 1 {
                                    return Err(parse::Error::new(
                                        span,
                                        "`extern` task required `#[task(..)]` attribute",
                                    ));
                                }

                                match crate::parse::task_args(
                                    item.attrs.remove(pos).tokens,
                                    settings,
                                )? {
                                    Either::Left(args) => {
                                        check_binding(&args.binds)?;
                                        check_ident(&item.sig.ident)?;

                                        hardware_tasks.insert(
                                            item.sig.ident.clone(),
                                            HardwareTask::parse_foreign(args, item)?,
                                        );
                                    }

                                    Either::Right(args) => {
                                        check_ident(&item.sig.ident)?;

                                        software_tasks.insert(
                                            item.sig.ident.clone(),
                                            SoftwareTask::parse_foreign(args, item)?,
                                        );
                                    }
                                }
                            } else {
                                return Err(parse::Error::new(
                                    span,
                                    "`extern` task required `#[task(..)]` attribute",
                                ));
                            }
                        } else {
                            return Err(parse::Error::new(
                                item.span(),
                                "this item must live outside the `#[app]` module",
                            ));
                        }
                    }
                }
                Item::Use(itemuse_) => {
                    // Store the user provided use-statements
                    user_imports.push(itemuse_.clone());
                }
                Item::Type(ref mut type_item) => {
                    // Match types with the attribute #[monotonic]
                    if let Some(pos) = type_item
                        .attrs
                        .iter()
                        .position(|attr| util::attr_eq(attr, "monotonic"))
                    {
                        let span = type_item.ident.span();

                        if monotonics.contains_key(&type_item.ident) {
                            return Err(parse::Error::new(
                                span,
                                "`#[monotonic(...)]` on a specific type must appear at most once",
                            ));
                        }

                        if type_item.vis != Visibility::Inherited {
                            return Err(parse::Error::new(
                                type_item.span(),
                                "this item must have inherited / private visibility",
                            ));
                        }

                        check_monotonic(&type_item.ty)?;

                        let m = type_item.attrs.remove(pos);
                        let args = MonotonicArgs::parse(m)?;

                        check_binding(&args.binds)?;

                        let monotonic = Monotonic::parse(args, type_item, span)?;

                        monotonics.insert(type_item.ident.clone(), monotonic);
                    }

                    // All types are passed on
                    user_code.push(item.clone());
                }
                _ => {
                    // Anything else within the module should not make any difference
                    user_code.push(item.clone());
                }
            }
        }

        let shared_resources_ident =
            shared_resources_ident.expect("No `#[shared]` resource struct defined");
        let local_resources_ident =
            local_resources_ident.expect("No `#[local]` resource struct defined");
        let init = init.expect("No `#[init]` function defined");

        if shared_resources_ident != init.user_shared_struct {
            return Err(parse::Error::new(
                init.user_shared_struct.span(),
                format!(
                    "This name and the one defined on `#[shared]` are not the same. Should this be `{}`?",
                    shared_resources_ident
                ),
            ));
        }

        if local_resources_ident != init.user_local_struct {
            return Err(parse::Error::new(
                init.user_local_struct.span(),
                format!(
                    "This name and the one defined on `#[local]` are not the same. Should this be `{}`?",
                    local_resources_ident
                ),
            ));
        }

        match (&actors_ident, &init.user_actors_struct) {
            (None, None) => {}
            (None, Some(init_actors_struct)) => {
                return Err(parse::Error::new(
                    init_actors_struct.span(),
                    format!(
                        "a `#[actors]` struct named `{}` needs to be defined",
                        init_actors_struct
                    ),
                ));
            }
            (Some(actors_ident), None) => {
                return Err(parse::Error::new(
                    actors_ident.span(),
                    format!(
                        "`#[init]` return type needs to include `{}` as the 4th tuple element ",
                        actors_ident
                    ),
                ));
            }
            (Some(actors_ident), Some(init_actors_struct)) => {
                if actors_ident != init_actors_struct {
                    return Err(parse::Error::new(
                        init_actors_struct.span(),
                        format!(
                            "This name and the one defined on `#[actors]` are not the same. Should this be `{}`?",
                            actors_ident
                        ),
                    ));
                }
            }
        }

        Ok(App {
            args,
            name: input.ident,
            init,
            idle,
            monotonics,
            shared_resources,
            local_resources,
            actors,
            user_imports,
            user_code,
            hardware_tasks,
            software_tasks,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::AppArgs;

    #[test]
    fn parse_app_args_true() {
        let s = "peripherals = true";

        let stream: proc_macro2::TokenStream = s.parse().unwrap();
        let result = AppArgs::parse(stream).unwrap();

        assert!(result.peripherals);
    }

    #[test]
    fn parse_app_args_false() {
        let s = "peripherals = false";

        let stream: proc_macro2::TokenStream = s.parse().unwrap();
        let result = AppArgs::parse(stream).unwrap();

        assert!(!result.peripherals);
    }

    #[test]
    fn parse_app_args_default() {
        let s = "";

        let stream: proc_macro2::TokenStream = s.parse().unwrap();
        let result = AppArgs::parse(stream).unwrap();

        assert!(result.peripherals);
    }
}
