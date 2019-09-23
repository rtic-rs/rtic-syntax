//! Full syntax for single core

#[mock::app(parse_binds)]
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

    #[task(binds = GPIOA)]
    fn foo1(ctx: foo1::Context) -> impl Generator<Yield = (), Return = !> {
        static mut X: u32 = 0;
        *X += 1;
    }

    // #[task(binds = GPIOB)]
    // fn foo2(ctx: foo1::Context) -> i32 {
    //     static mut X: u32 = 0;
    //     *X += 1;
    // }

    // fn foo(ctx: Context) -> Foo {
    //     // let x = cx.resources.x;

    //     // move || loop {
    //     //     // x.lock(|_| {});
    //     //     yield
    //     // }
    // }

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
};
