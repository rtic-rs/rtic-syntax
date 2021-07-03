#![no_main]

#[mock::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        a: u32,
    }

    #[task(local = [a])]
    fn foo(_: foo::Context) {}

    #[task(local = [a: u8 = 3])]
    fn bar(_: bar::Context) {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}
}
