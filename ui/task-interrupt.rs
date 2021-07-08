#![no_main]

#[mock::app(parse_binds)]
mod app {
    #[task(binds = SysTick)]
    fn foo(_: foo::Context) {}

    #[task]
    fn foo(_: foo::Context) {}
}
