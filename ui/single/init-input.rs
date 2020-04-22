#![no_main]

#[mock::app]
mod app {
    #[init]
    fn init(_: init::Context, _undef: u32) {}
}
