#![no_main]

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        // An exclusive, early resource
        #[lock_free]
        #[init(1)]
        e1: u32,

        // An exclusive, late resource
        #[lock_free]
        e2: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {}

    // e2 ok
    #[idle(resources =[e2])]
    fn idle(cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);
        loop {}
    }

    // e1 rejected (not lock_free)
    #[task(priority = 1, resources = [e1])]
    fn uart0(cx: uart0::Context) {
        *cx.resources.e1 += 10;
    }

    // e1 rejected (not lock_free)
    #[task(priority = 2, resources = [e1])]
    fn uart1(cx: uart1::Context) {}
}
