#![no_main]

#[mock::app(parse_interrupt)]
const APP: () = {
    #[interrupt(binds = UART0)]
    fn foo(_: foo::Context) {}

    #[interrupt(binds = UART0)]
    fn bar(_: bar::Context) {}
};
