#![no_main]

#[mock::app]
mod app {
    #[idle]
    pub fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
