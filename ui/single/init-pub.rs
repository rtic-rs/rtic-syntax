#![no_main]

#[mock::app]
const APP: () = {
    #[init]
    pub fn init(_: init::Context) {}
};
