#![no_main]

#[mock::app]
mod app {
    #[idle]
    unsafe fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
