#![no_main]

#[mock::app]
const APP: () = {
    #[task]
    fn init(_: init::Context) {}
};
