#![no_main]

#[mock::app(cores = 2, parse_cores, parse_binds)]
const APP: () = {
    #[task(core = 0, binds = UART0)]
    fn foo(_: foo::Context) {}

    #[task(core = 0, binds = UART0)]
    fn bar(_: bar::Context) {}
};
