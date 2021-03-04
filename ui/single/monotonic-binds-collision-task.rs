#![no_main]

#[mock::app(parse_extern_interrupt, parse_binds)]
mod app {
    #[monotonic(binds = Tim1)]
    type Fast1 = hal::Tim1Monotonic;

    #[task(binds = Tim1)]
    fn foo(_: foo::Context) {}
}
