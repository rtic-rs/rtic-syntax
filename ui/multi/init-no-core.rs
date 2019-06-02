#![no_main]

#[mock::app(cores = 2, parse_cores)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {}
};
