#![no_main]

#[mock::app]
mod app {
    #[task(binds = UART0)]
    fn foo(_: foo::Context) {}
}
