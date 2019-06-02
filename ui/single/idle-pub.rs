#![no_main]

#[mock::app]
const APP: () = {
    #[idle]
    pub fn idle(_: idle::Context) -> ! {
        loop {}
    }
};
