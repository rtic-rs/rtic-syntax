#![no_main]

#[mock::app]
mod app {
    struct Resources {
        x: u32,
    }

    #[init(resources = [x])]
    fn init(_: init::Context) {}
}
