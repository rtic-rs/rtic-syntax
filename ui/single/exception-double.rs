#![no_main]

#[mock::app(parse_exception)]
const APP: () = {
    #[exception(binds = UART0)]
    fn foo(_: foo::Context) {}

    #[exception(binds = UART0)]
    fn bar(_: bar::Context) {}
};
