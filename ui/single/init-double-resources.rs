#![no_main]

#[mock::app]
const APP: () = {
    #[init(resources = [A], resources = [B])]
    fn init(_: init::Context) {}
};
