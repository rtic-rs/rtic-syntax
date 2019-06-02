#![no_main]

#[mock::app]
const APP: () = {
    extern "C" {
        static X: u32;
    }

    #[init]
    fn init(_: init::Context) {}
};
