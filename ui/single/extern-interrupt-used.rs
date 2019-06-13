#![no_main]

#[mock::app(parse_extern_interrupt, parse_interrupt)]
const APP: () = {
    #[interrupt(binds = EXTI0)]
    fn foo(_: foo::Context) {}

    extern "C" {
        fn EXTI0();
    }
};
