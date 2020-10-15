#![no_main]

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        // A local (move), early resource
        #[cfg(feature = "feature_l1")]
        #[task_local]
        #[init(1)]
        l1: u32,

        // A local (move), late resource
        #[task_local]
        l2: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        init::LateResources {
            #[cfg(feature = "feature_l2")]
            l2: 2,
            #[cfg(not(feature = "feature_l2"))]
            l2: 5,
        }
    }

    // l1 ok (task_local)
    #[idle(resources =[#[cfg(feature = "feature_l1")]l1])]
    fn idle(_cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);
        loop {}
    }

    // l2 ok (task_local)
    #[task(priority = 1, resources = [
        #[cfg(feature = "feature_l2")]l2,
    ])]
    fn uart0(_cx: uart0::Context) {
    }

    // l2 error, conflicting with uart0 for l2 (task_local)
    #[task(priority = 1, resources = [
        #[cfg(not(feature = "feature_l2"))]l2
    ])]
    fn uart1(_cx: uart1::Context) {
    }
}
