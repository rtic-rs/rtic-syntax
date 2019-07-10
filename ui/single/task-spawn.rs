#![no_main]

#[mock::app(parse_binds)]
const APP: () = {
    #[task(binds = FOO)]
    fn foo(_: foo::Context) {}

    #[task(spawn = [foo])]
    fn bar(_: bar::Context) {}
};
