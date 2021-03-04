#![no_main]

#[mock::app]
mod app {
    #[monotonic()]
    type Fast = hal::Tim1Monotonic;
}
