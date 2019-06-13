#![no_main]

#[mock::app(parse_extern_interrupt, parse_interrupt)]
const APP: () = {
    #[interrupt]
    fn EXTI0(_: EXTI0::Context) {}

    extern "C" {
        fn EXTI0();
    }
};
