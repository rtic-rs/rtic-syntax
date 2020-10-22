#![no_main]

#[mock::app(parse_extern_interrupt, parse_binds, dispatchers = [EXTI0])]
mod app {
    #[task(binds = EXTI0)]
    fn foo(_: foo::Context) {}
}
