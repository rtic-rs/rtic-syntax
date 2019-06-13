#![no_main]

#[mock::app(cores = 2, parse_cores, parse_exception)]
const APP: () = {
    #[exception(core = 0, binds = UART0)]
    fn foo(_: foo::Context) {}

    #[exception(core = 0, binds = UART0)]
    fn bar(_: bar::Context) {}
};
