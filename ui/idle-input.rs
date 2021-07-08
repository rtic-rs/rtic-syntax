#![no_main]

#[mock::app]
mod app {
    #[idle]
    fn idle(_: idle::Context, _undef: u32) -> ! {
        loop {}
    }
}
