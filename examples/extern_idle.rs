//! examples/extern_idle

#[mock::app(parse_binds, dispatchers = [UART1])]
mod app {
    // idle externally implemented
    use crate::{bar, foo};

    #[shared]
    struct Shared {
        a: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    extern "Rust" {
        // Software task
        #[task(shared = [a], priority = 2)]
        fn foo(_: foo::Context, _: u32);

        // Hardware task
        #[task(binds = UART0, shared = [a], priority = 2)]
        // #[inline(always)] // would be rejected
        fn bar(_: bar::Context);

        // Externally defined idle task
        #[idle()]
        fn idle(_: idle::Context) -> !;
    }
}

// The actual functions to dispatch are
// defined outside of the mod `app`.
//
// fn foo(_: foo::Context, _: u32) {}
// fn bar(_: bar::Context, _: u32) {}
// fn idle(_: idle::Context) -> ! {}
