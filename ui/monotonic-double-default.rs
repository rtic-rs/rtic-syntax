#![no_main]

#[mock::app]
mod app {
    #[monotonic(binds = Tim1, default = true, default = false)]
    type Fast = hal::Tim1Monotonic;
}
