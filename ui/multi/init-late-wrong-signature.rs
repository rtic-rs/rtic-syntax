#![no_main]

#[mock::app(cores = 2, parse_cores)]
const APP: () = {
    extern "C" {
        static A: u32;
    }

    #[init(core = 0, late = [A])]
    fn init(_: init::Context) {}

    #[init(core = 1)]
    fn init(_: init::Context) {}
};
