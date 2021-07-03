#![no_main]

#[mock::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    pub fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}
}
