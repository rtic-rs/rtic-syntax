#![no_main]

#[mock::app(cores = 2, parse_cores)]
const APP: () = {
    struct Resources {
        a: u32,
        #[init(0)]
        b: u32,
    }

    #[init(core = 0, late = [b])]
    fn init(_: init::Context) -> init::LateResources {}
};
