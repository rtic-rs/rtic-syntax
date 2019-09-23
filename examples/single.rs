//! Full syntax for single core

#[mock::app(parse_binds, parse_impl_generator)]
const APP: () = {
    struct Resources {
        a: u32,
        b: u32,
        #[init(0)]
        c: u32,
        #[init(0)]
        d: u32,
    }

    #[init(
        resources = [c],
        spawn = [foo],
    )]
    fn init(_: init::Context) -> init::LateResources {
        #[cfg(debug_assertions)]
        static mut X: u32 = 0;

        init::LateResources { a: 0, b: 0 }
    }

    #[idle(
        resources = [&a, d],
        spawn = [foo],
    )]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[task(
        resources = [b, &c],
        spawn = [bar],
    )]
    fn foo(_: foo::Context) {
        static mut X: u32 = 0;

        *X += 1;
    }

    #[task(
        capacity = 2,
        priority = 2,
        resources = [d],
        spawn = [foo],
    )]
    fn bar(_: bar::Context, _: u32) {
        static mut X: u32 = 0;

        *X += 1;
    }

    #[task(binds = EXTI0)]
    fn baz(_: baz::Context) -> impl Generator<Yield = (), Return = !> {
        // ..
    }
};
