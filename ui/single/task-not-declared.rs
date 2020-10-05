#![no_main]

#[mock::app]
mod app {
    #[init(spawn = [foo])]
    fn init(_: init::Context) -> init::LateResources {}
}
