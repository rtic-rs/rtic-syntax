//! Full syntax for multi core

#[mock::app(cores = 2, parse_cores, parse_exception, parse_interrupt)]
const APP: () = {
    extern "C" {
        static A: u32;
        static mut B: u32;
    }

    static C: u32 = 0;
    static mut D: u32 = 0;

    #[init(core = 0, late = [A], spawn = [foo, bar])]
    fn init(_: init::Context) -> init::LateResources {
        #[cfg(debug_assertions)]
        static mut X: u32 = 0;

        init::LateResources { A: 0 }
    }

    #[init(core = 1, spawn = [foo, bar])]
    fn init(_: init::Context) -> init::LateResources {
        static mut X: u32 = 0;

        init::LateResources { B: 0 }
    }

    #[idle(core = 0, spawn = [foo, bar])]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[idle(core = 1, spawn = [foo, bar])]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[task(core = 0, spawn = [bar])]
    fn foo(_: foo::Context, _: u32) {
        static mut X: u32 = 0;
    }

    #[task(core = 1, spawn = [foo])]
    fn bar(_: bar::Context) {
        static mut X: u32 = 0;
    }

    #[exception(core = 0, binds = SysTick)]
    fn e0(_: e0::Context) {}

    #[exception(core = 1, binds = SysTick)]
    fn e1(_: e1::Context) {}

    #[interrupt(core = 0, binds = UART0)]
    fn i0(_: i0::Context) {}

    #[interrupt(core = 1, binds = UART0)]
    fn i1(_: i1::Context) {}
};
