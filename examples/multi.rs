//! Full syntax for multi core

#[mock::app(cores = 2, parse_cores, parse_binds)]
const APP: () = {
    struct Resources {
        a: u32,
        b: u32,
        #[init(0)]
        c: u32,
        #[init(0)]
        d: u32,
    }

    #[init(core = 0, late = [a], spawn = [foo, bar])]
    fn init(_: init::Context) -> init::LateResources {
        #[cfg(debug_assertions)]
        static mut X: u32 = 0;

        init::LateResources { a: 0 }
    }

    #[init(core = 1, spawn = [foo, bar])]
    fn init(_: init::Context) -> init::LateResources {
        static mut X: u32 = 0;

        init::LateResources { b: 0 }
    }

    #[idle(core = 0, spawn = [foo, bar], resources = [&a, b, &c])]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[idle(core = 1, spawn = [foo, bar], resources = [&a, &c, d])]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[task(core = 0, spawn = [bar], resources = [b])]
    fn foo(_: foo::Context, _: u32) {
        static mut X: u32 = 0;
    }

    #[task(core = 1, spawn = [foo], resources = [d])]
    fn bar(_: bar::Context) {
        static mut X: u32 = 0;
    }

    #[task(core = 0, binds = SysTick)]
    fn e0(_: e0::Context) {}

    #[task(core = 1, binds = SysTick)]
    fn e1(_: e1::Context) {}

    #[task(core = 0, binds = UART0)]
    fn i0(_: i0::Context) {}

    #[task(core = 1, binds = UART0)]
    fn i1(_: i1::Context) {}
};
