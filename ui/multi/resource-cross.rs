#![no_main]

#[mock::app(cores = 2, parse_cores)]
const APP: () = {
    static mut X: u32 = 0;

    #[task(core = 0, resources = [X])]
    fn foo(_: foo::Context) {}

    #[task(core = 1, resources = [X])]
    fn bar(_: bar::Context) {}
};
