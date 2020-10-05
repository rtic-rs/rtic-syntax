#![no_main]

#[mock::app]
mod app {
    #[idle(spawn = [foo], spawn = [bar])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
