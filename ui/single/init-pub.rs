#![no_main]

#[mock::app]
mod app {
    #[init]
    pub fn init(_: init::Context) {}
}
