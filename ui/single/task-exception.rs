#![no_main]

#[mock::app(parse_exception)]
const APP: () = {
    #[exception]
    fn foo(_: foo::Context) {}

    // name collides with `#[idle]` function
    #[task]
    fn foo(_: foo::Context) {}
};
