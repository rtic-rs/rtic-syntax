#![no_main]

#[mock::app(cores = 2, parse_cores)]
const APP: () = {
    struct Resources {
        a: u32,
        b: u32,
    }

    #[init(core = 0, late = [a])]
    fn init(_: init::Context) -> init::LateResources {}

    #[init(core = 1, late = [a])]
    fn init(_: init::Context) -> init::LateResources {}
};
