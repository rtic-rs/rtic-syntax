//! Full syntax

#[mock::app]
mod app {
    #[shared]
    struct Shared {
        a: u32,
        b: u32,
        c: u32,
        d: u32,
    }

    #[local]
    struct Local {
        a: u32,
        b: u32,
        c: u32,
        d: u32,
    }

    #[init(local = [a: u32 = 3, b: u8 = 2])]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        init::LateResources { }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    #[task(shared = [a], local = [b])]
    fn t1(_: t1::Context) {
    }

    #[task(shared = [a], local = [b: (u32, core::u8) = (4, 3)])]
    fn t2(_: t2::Context) {
    }

    // #[idle]
    // fn idle(_: idle::Context) -> ! {

    //     loop {}
    // }

    // #[resources]
    // struct Resources {
    //     a: u32,
    //     b: u32,
    //     #[init(0)]
    //     c: u32,
    //     #[init(0)]
    //     d: u32,
    // }

    // #[monotonic(binds = Tim1)]
    // type Fast = hal::Tim1Monotonic;

    // fn test() {}

    // #[init(
    //     resources = [c],
    // )]
    // fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
    //     init::LateResources { a: 0, b: 0 }
    // }

    // #[idle(
    //     resources = [&a, d],
    // )]
    // fn idle(_: idle::Context) -> ! {

    //     loop {}
    // }

    // #[task(
    //     resources = [b, &c],
    // )]
    // fn foo(_: foo::Context) {
    // }

    // #[task(
    //     capacity = 2,
    //     priority = 2,
    //     resources = [d],
    // )]
    // fn bar(_: bar::Context, _: u32) {
    // }
}
