#![no_main]

#[mock::app]
const APP: () = {
    #[init]
    unsafe fn init(_: init::Context) {}
};
