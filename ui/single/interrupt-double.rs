#![no_main]

#[mock::app(parse_binds)]
const APP: () = {
    #[task(binds = UART0)]
    fn foo(_: foo::Context) {}

    #[task(binds = UART0)]
    fn bar(_: bar::Context) {}
};
