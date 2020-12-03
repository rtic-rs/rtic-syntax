#![no_main]

#[mock::app]
mod app {
    #[monotonic(binds = Tim1)]
    type Fast = hal::Tim1Monotonic;
}
