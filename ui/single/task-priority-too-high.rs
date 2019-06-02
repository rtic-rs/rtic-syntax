#![no_main]

#[mock::app]
const APP: () = {
    #[task(priority = 256)]
    fn foo(_: foo::Context) {}
};
