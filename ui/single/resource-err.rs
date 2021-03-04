#![no_main]

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        // An early resource
        #[init(0)]
        shared: u32,

        // A local (move), early resource
        #[task_local]
        #[init(1)]
        l1: u32,

        // An exclusive, early resource
        #[lock_free]
        #[init(1)]
        e1: u32,

        // A local (move), late resource
        #[task_local]
        l2: u32,

        // An exclusive, late resource
        #[lock_free]
        e2: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {}

    // `shared` cannot be accessed from this context
    // l1 ok
    // l2 rejeceted (not task_local)
    // e2 ok
    #[idle(resources =[l1, l2, e2])]
    fn idle(cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);
        loop {}
    }

    // `shared` can be accessed from this context
    // l2 rejected (not task_local)
    // e1 rejected (not lock_free)
    #[task(priority = 1, resources = [shared, l2, e1])]
    fn uart0(cx: uart0::Context) {
        let shared: &mut u32 = cx.resources.shared;
        *shared += 1;
        *cx.resources.e1 += 10;
    }

    // l2 rejected (not task_local)
    #[task(priority = 2, resources = [shared, l2, e1])]
    fn uart1(cx: uart1::Context) {
        let shared: &mut u32 = cx.resources.shared;
        *shared += 1;
    }
}
