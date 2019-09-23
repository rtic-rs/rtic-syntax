#![no_main]

#[mock::app(parse_binds)]
const APP: () = {
    #[task(binds = EXTI0)]
    fn foo(_: foo::Context) -> impl Generator<Yield = (), Return = !> {
        // ..
    }
};
