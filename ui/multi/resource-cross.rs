#![no_main]

#[mock::app(cores = 2, parse_cores)]
const APP: () = {
    struct Resources {
        x: u32,
    }

    #[task(core = 0, resources = [x])]
    fn foo(_: foo::Context) {}

    #[task(core = 1, resources = [x])]
    fn bar(_: bar::Context) {}
};
