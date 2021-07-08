#![no_main]

#[mock::app]
mod app {
    #[idle(shared = [A], shared = [B])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
