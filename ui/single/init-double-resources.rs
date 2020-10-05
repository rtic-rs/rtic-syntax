#![no_main]

#[mock::app]
mod app {
    #[init(resources = [A], resources = [B])]
    fn init(_: init::Context) {}
}
