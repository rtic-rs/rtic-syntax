#![no_main]

#[mock::app]
mod app {
    #[init]
    unsafe fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {}
}
