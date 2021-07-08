#![no_main]

#[mock::app]
mod app {
    #[idle]
    fn idle() -> ! {
        loop {}
    }
}
