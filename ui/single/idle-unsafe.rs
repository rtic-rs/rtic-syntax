#![no_main]

#[mock::app]
const APP: () = {
    #[idle]
    unsafe fn idle(_: idle::Context) -> ! {
        loop {}
    }
};
