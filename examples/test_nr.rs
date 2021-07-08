//! Full syntax

#[mock::app(parse_binds,
    dispatchers = [
        #[link_section = ".data.UART1"]
        A,
        #[link_section = ".data.UART2"]
        B
    ])
]
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

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    #[task]
    fn t1(_: t1::Context) {}

    #[task(local = [
        #[testing1]
        #[testing2]
        #[link_section = ".example_section"]
        q: (u32, core::u8) = (4, 3),
    ])]
    fn t2(_: t2::Context) {}

    #[task(local = [ohno: u32 = 0])]
    fn t3(_: t3::Context) {}

    #[task(local = [ohno: u32 = 0])]
    fn t4(_: t4::Context) {}
}
