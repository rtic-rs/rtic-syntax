#![no_main]

#[mock::app]
mod app {
    #[init(local = [A], local = [B])]
    fn init(_: init::Context) {}
}
