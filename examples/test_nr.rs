//! Full syntax

#[mock::app]
mod app {
    #[shared]
    struct Shared {
        a: u32,
        b: u32,
        c: u32,
        d: u32,
    }

    #[local]
    struct Local {
        a: u32,
        b: u32,
        c: u32,
        d: u32,
    }

    #[init(local = [a: u32 = 3, b: u8 = 2])]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    #[task(shared = [a], local = [b])]
    fn t1(_: t1::Context) {}

    #[task(shared = [a], local = [q: (u32, core::u8) = (4, 3)])]
    fn t2(_: t2::Context) {}
}
