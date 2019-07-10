#![no_main]

#[mock::app]
const APP: () = {
    struct Resources {
        x: u32,
    }

    #[init]
    fn init(_: init::Context) {}
};
