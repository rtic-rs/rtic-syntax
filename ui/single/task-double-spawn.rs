#![no_main]

#[mock::app]
mod app {
    #[task(spawn = [bar], spawn = [baz])]
    fn foo(_: foo::Context) {}
}
