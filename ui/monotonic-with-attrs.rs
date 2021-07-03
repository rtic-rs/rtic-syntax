#![no_main]

#[mock::app]
mod app {
    #[no_mangle]
    #[monotonic(binds = Tim1)]
    type Fast = hal::Tim1Monotonic;
}
