#![no_main]

#[mock::app]
mod app {
    #[idle(local = [A], local = [B])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
