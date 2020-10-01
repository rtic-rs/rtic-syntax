#![no_main]

#[mock::app]
const APP: () = {
    struct Resources {
        x: u32,
    }

    #[init(resources = [x])]
    fn init(_: init::Context) -> init::LateResources {}
};
