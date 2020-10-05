#![no_main]

#[mock::app]
mod app {
    #[init(spawn = [foo], spawn = [bar])]
    fn init(_: init::Context) {}
}
