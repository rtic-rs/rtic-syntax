//! Raw `Synom` parsing

use either::Either;
use proc_macro2::TokenStream;
use syn::Path;
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Ident, LitInt, LitBool};

pub struct Map<KV>
where
    KV: Synom,
{
    pub kvs: Punctuated<KV, Token![,]>,
}

pub type App = Map<AppKv>;
pub type Idle = Map<IdleKv>;
pub type Task = Map<TaskKv>;
pub type Tasks = Map<TasksKv>;

impl<KV> Synom for Map<KV>
where
    KV: Synom,
{
    named!(parse -> Self, map!(call!(Punctuated::parse_terminated), |kvs| Map { kvs }));
}

/// Key value pair inside `app! { .. }`
///
/// `$key:ident: $value:path` OR `$key:ident: { $($value:tt)* }`
pub struct AppKv {
    pub key: Ident,
    pub value: Either<Path, TokenStream>,
}

impl Synom for AppKv {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
            _colon: punct!(:) >>
            value: alt!(
                map!(syn!(Path), |path| Either::Left(path)) |
                map!(braces!(syn!(TokenStream)), |(_, ts)| Either::Right(ts))
            ) >>
            (AppKv { key, value })
    ));
}

pub struct IdleKv {
    pub key: Ident,
    pub value: Either<Path, TokenStream>,
}

impl Synom for IdleKv {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
            _colon: punct!(:) >>
            value: alt!(
                map!(syn!(Path), |path| Either::Left(path)) |
                map!(brackets!(syn!(TokenStream)), |(_, ts)| Either::Right(ts))
            ) >>
            (IdleKv { key, value })
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
    Bool(LitBool),
    Idents(TokenStream),
    Int(LitInt),
    Path(Path),
}

pub struct TaskKv {
    pub key: Ident,
    pub value: TaskValue,
}

impl Synom for TaskKv {
    named!(parse -> Self, do_parse!(
        key: syn!(Ident) >>
            _colon: punct!(:) >>
            value: alt!(
                map!(syn!(LitBool), |lb| TaskValue::Bool(lb)) |
                map!(syn!(LitInt), |li| TaskValue::Int(li)) |
                map!(syn!(Path), |path| TaskValue::Path(path)) |
                map!(brackets!(syn!(TokenStream)), |(_, ts)| TaskValue::Idents(ts))
            ) >>
            (TaskKv { key, value })
    ));
}
