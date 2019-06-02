#![no_main]

#[mock::app]
const APP: () = {
    #[init]
    fn init(_: init::Context) -> u32 {
        0
    }
};
