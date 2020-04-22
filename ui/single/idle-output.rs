#![no_main]

#[mock::app]
mod app {
    #[idle]
    fn idle(_: idle::Context) -> u32 {
        0
    }
}
