#![no_main]

#[mock::app]
mod app {
    #[idle(resources = [A], resources = [B])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
