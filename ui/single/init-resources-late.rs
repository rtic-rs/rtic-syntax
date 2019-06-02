#![no_main]

#[mock::app]
const APP: () = {
    extern "C" {
        static mut X: u32;
    }

    #[init(resources = [X])]
    fn init(_: init::Context) {}
};
