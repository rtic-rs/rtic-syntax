#![no_main]

#[mock::app(parse_binds, parse_impl_generator)]
const APP: () = {
    #[task(binds = EXTI0)]
    fn foo(_: foo::Context) -> impl Generator<Yield = (), Return = ()> {
        // ..
    }
};
