//! Test async

#[mock::app(parse_binds, dispatchers = [UART1])]
mod app {
    #[init]
    fn init(_: init::Context) -> init::LateResources {
        init::LateResources {}
    }

    #[idle]
    async fn idle(_: idle::Context) -> ! {}

    #[task(binds = UART0)]
    async fn foo(_: foo::Context) {}

    #[task()]
    async fn bar(_: bar::Context) {}
}
