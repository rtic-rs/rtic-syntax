#![no_main]

#[mock::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[task(local = [a:])]
    fn foo(_: foo::Context) {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}
}
