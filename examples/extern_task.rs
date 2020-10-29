//! examples/extern_task

#[mock::app(parse_binds, dispatchers = [UART1])]
mod app {
    // task externally implemented
    use crate::{bar, foo};
    #[resources]
    struct Resources {
        a: u32,
    }

    #[init()]
    fn init(_: init::Context) -> init::LateResources {
        init::LateResources {}
    }

    #[idle()]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    extern "Rust" {
        // Software task
        #[task(resources = [a], priority = 2)]
        fn foo(_: foo::Context, _: u32);

        // Hardware task
        #[task(binds = UART0, resources = [a], priority = 2)]
        // #[inline(always)] // would be rejected
        fn bar(_: bar::Context);
    }
}

// The actual functions to dispatch are
// defined outside of the mod `app`.
//
// fn foo(_: foo::Context, _: u32) {}
// fn bar(_: bar::Context, _: u32) {}
