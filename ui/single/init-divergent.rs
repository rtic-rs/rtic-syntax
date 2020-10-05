#![no_main]

#[mock::app]
mod app {
    #[init]
    fn init(_: init::Context) -> ! {
        loop {}
    }
}
