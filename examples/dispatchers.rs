//! Test binds

#[mock::app(parse_binds,
    dispatchers = [
        #[link_section = ".data.UART1"]
        A,
        #[link_section = ".data.UART2"]
        B
    ])
]
mod app {
    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {}

    #[task(binds = UART0)]
    fn foo(_: foo::Context) {}
}
