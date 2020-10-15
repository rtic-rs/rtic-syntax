//! examples/local.rs

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        a: u32,
        b: u32,
        #[init(0)]
        c: u32,
        #[init(0)]
        d: u32,
        #[task_local]
        tl: u32,
        #[task_local]
        #[init(0)]
        tl_init: u32,
        #[lock_free]
        lf: u32,
    }

    #[init(
                resources = [c],
            )]
    fn init(_: init::Context) -> init::LateResources {
        #[cfg(debug_assertions)]
        static mut X: u32 = 0;

        init::LateResources { a: 0, b: 0 }
    }

    #[idle(
                resources = [&a, d, lf],
            )]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[task(
                resources = [b, &c],
            )]
    fn foo(_: foo::Context) {
        static mut X: u32 = 0;

        *X += 1;
    }

    #[task(
                capacity = 2,
                priority = 2,
                resources = [d],
            )]
    fn bar(_: bar::Context, _: u32) {
        static mut X: u32 = 0;

        *X += 1;
    }
    #[task(
                resources = [tl, tl_init],
            )]
    fn task_local_fn(_: task_local_fn::Context) {}
}
