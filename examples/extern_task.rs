//! examples/extern_task

#[mock::app(parse_binds, dispatchers = [UART1])]
mod app {
    // task externally implemented
    use crate::{bar, foo};

    #[shared]
    struct Shared {
        a: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    #[idle]
    fn idle(_: idle::Context) -> ! {}

    extern "Rust" {
        // Software task
        #[task(shared = [a], priority = 2)]
        fn foo(_: foo::Context, _: u32);

        // Hardware task
        #[task(binds = UART0, shared = [a], priority = 2)]
        // #[inline(always)] // would be rejected
        fn bar(_: bar::Context);
    }
}

// The actual functions to dispatch are
// defined outside of the mod `app`.
//
// fn foo(_: foo::Context, _: u32) {}
// fn bar(_: bar::Context, _: u32) {}
