//! Full syntax for multi core

#[mock::app(cores = 2, parse_cores)]
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
};
