#![no_main]

#[mock::app]
const APP: () = {
    #[init(spawn = [foo])]
    fn init(_: init::Context) -> init::LateResources {}
};
