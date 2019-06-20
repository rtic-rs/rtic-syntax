#![no_main]

#[mock::app(parse_binds)]
const APP: () = {
    #[task(binds = SysTick)]
    fn foo(_: foo::Context) {}

    #[task]
    fn foo(_: foo::Context) {}
};
