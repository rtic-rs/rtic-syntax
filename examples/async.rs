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
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

    #[idle]
    fn idle(_: idle::Context) -> ! {}

    #[task]
    async fn foo(_: foo::Context) {}

    #[task]
    fn foo2(_: foo2::Context) {}
}
