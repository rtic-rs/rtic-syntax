//! Raw `Synom` parsing

use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::token::{Brace, Bracket};
use syn::Path;
use syn::{Ident, LitInt, Type};

pub struct Map<KV>
where
    KV: Synom,
{
    pub kvs: Punctuated<KV, Token![,]>,
}

pub type App = Map<AppKv>;
pub type Init = Map<InitKv>;
pub type Task = Map<TaskKv>;
pub type Tasks = Map<TasksKv>;

impl<KV> Synom for Map<KV>
where
    KV: Synom,
{
    named!(parse -> Self, map!(call!(Punctuated::parse_terminated), |kvs| Map { kvs }));
}

pub struct AppKv {
    pub key: Ident,
    pub value: AppValue,
}

pub enum AppValue {
    Device(Path),
    Idle((Brace, TokenStream)),
    Init((Brace, TokenStream)),
    Resources((Brace, TokenStream)),
    FreeInterrupts((Bracket, TokenStream)),
    Tasks((Brace, TokenStream)),
    Unknown(TokenStream),
}

impl Synom for AppKv {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
            _colon: punct!(:) >>
            value: switch!(
                value!(key.as_ref()),
                "device" => map!(syn!(Path), AppValue::Device) |
                "idle" => map!(braces!(syn!(TokenStream)), AppValue::Idle) |
                "init" => map!(braces!(syn!(TokenStream)), AppValue::Init) |
                "resources" => map!(braces!(syn!(TokenStream)), AppValue::Resources) |
                "free_interrupts" => map!(brackets!(syn!(TokenStream)), AppValue::FreeInterrupts) |
                "tasks" => map!(braces!(syn!(TokenStream)), AppValue::Tasks) |
                _ => map!(syn!(TokenStream), AppValue::Unknown)
            ) >>
            (AppKv { key, value })
    ));
}

pub struct InitKv {
    pub key: Ident,
    pub value: InitValue,
}

pub enum InitValue {
    Path(Path),
    Resources((Bracket, TokenStream)),
    ScheduleNow((Bracket, TokenStream)),
    ScheduleAfter((Bracket, TokenStream)),
    Unknown(TokenStream),
}

impl Synom for InitKv {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
            _colon: punct!(:) >>
            value: switch!(
                value!(key.as_ref()),
                "path" => map!(syn!(Path), InitValue::Path) |
                "resources" => map!(brackets!(syn!(TokenStream)), InitValue::Resources) |
                "schedule_now" => map!(brackets!(syn!(TokenStream)), InitValue::ScheduleNow) |
                "schedule_after" => map!(brackets!(syn!(TokenStream)), InitValue::ScheduleAfter) |
                _ => map!(syn!(TokenStream), InitValue::Unknown)
            ) >>
            (InitKv { key, value })
    ));
}

pub struct TasksKv {
    pub key: Ident,
    pub value: TokenStream,
}

impl Synom for TasksKv {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
            _colon: punct!(:) >>
            value: map!(braces!(syn!(TokenStream)), |(_, ts)| ts) >>
            (TasksKv { key, value })
    ));
}

pub enum TaskValue {
    Interrupt(Ident),
    Path(Path),
    Input(Type),
    ScheduleNow((Bracket, TokenStream)),
    ScheduleAfter((Bracket, TokenStream)),
    Resources((Bracket, TokenStream)),
    Priority(LitInt),
    Instances(LitInt),
    Unknown(TokenStream),
}

pub struct TaskKv {
    pub key: Ident,
    pub value: TaskValue,
}

impl Synom for TaskKv {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
            _colon: punct!(:) >>
            value: switch!(
                value!(key.as_ref()),
                "interrupt" => map!(syn!(Ident), TaskValue::Interrupt) |
                "path" => map!(syn!(Path), TaskValue::Path) |
                "input" => map!(syn!(Type), TaskValue::Input) |
                "type" => map!(syn!(Type), TaskValue::Input) |
                "schedule_now" => map!(brackets!(syn!(TokenStream)), TaskValue::ScheduleNow) |
                "schedule_after" => map!(brackets!(syn!(TokenStream)), TaskValue::ScheduleAfter) |
                "resources" => map!(brackets!(syn!(TokenStream)), TaskValue::Resources) |
                "priority" => map!(syn!(LitInt), TaskValue::Priority) |
                "instances" => map!(syn!(LitInt), TaskValue::Instances) |
                _ => map!(syn!(TokenStream), TaskValue::Unknown)
            ) >>
            (TaskKv { key, value })
    ));
}
