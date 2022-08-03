#![no_main]

#[mock::app(parse_binds)]
mod app {
    #[task(binds = UART0, priority = 0)]
    fn foo(_: foo::Context) {}
}
