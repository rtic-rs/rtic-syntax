#![no_main]

#[mock::app]
const APP: () = {
    #[init(spawn = [foo], spawn = [bar])]
    fn init(_: init::Context) {}
};
