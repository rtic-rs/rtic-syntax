#![no_main]

#[mock::app(parse_exception, parse_interrupt)]
const APP: () = {
    #[exception(binds = UART0)]
    fn foo(_: foo::Context) {}

    #[interrupt(binds = UART0)]
    fn bar(_: bar::Context) {}
};
