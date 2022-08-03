#![no_main]

#[mock::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    #[task(priority = 0)]
    fn foo(_: foo::Context) {}

    #[idle]
    fn idle(_: idle::Context) -> ! {}
}
