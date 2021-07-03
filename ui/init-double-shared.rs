#![no_main]

#[mock::app]
mod app {
    #[init(shared = [A], shared = [B])]
    fn init(_: init::Context) {}
}
