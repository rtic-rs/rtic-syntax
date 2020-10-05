#![no_main]

#[mock::app(parse_binds)]
mod app {
    #[task(binds = FOO)]
    fn foo(_: foo::Context) {}

    #[task(spawn = [foo])]
    fn bar(_: bar::Context) {}
}
