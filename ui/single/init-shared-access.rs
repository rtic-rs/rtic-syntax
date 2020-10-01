#![no_main]

#[mock::app]
const APP: () = {
    struct Resources {
        #[init(0)]
        x: i32,
    }

    #[init(resources = [&x])]
    fn init(_: init::Context) -> init::LateResources {}
};
