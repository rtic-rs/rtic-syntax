#![no_main]

#[mock::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context, _undef: u32) -> (Shared, Local, init::Monotonics) {}
}
