//! Full syntax for single core

#[mock::app]
const APP: () = {
    // late resources
    extern "C" {
        static A: u32;
        static mut B: u32;
    }

    // early resources
    static C: u32 = 0;
    static mut D: u32 = 0;

    #[init(
        resources = [C],
        spawn = [foo],
    )]
    fn init(_: init::Context) -> init::LateResources {
        #[cfg(debug_assertions)]
        static mut X: u32 = 0;

        init::LateResources { A: 0, B: 0 }
    }

    #[idle(
        resources = [D],
        spawn = [foo],
    )]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[task(
        resources = [C],
        spawn = [bar],
    )]
    fn foo(_: foo::Context) {
        static mut X: u32 = 0;

        *X += 1;
    }

    #[task(
        capacity = 2,
        priority = 2,
        resources = [D],
        spawn = [foo],
    )]
    fn bar(_: bar::Context, _: u32) {
        static mut X: u32 = 0;

        *X += 1;
    }
};
