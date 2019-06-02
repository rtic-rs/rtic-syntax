#![no_main]

#[mock::app]
const APP: () = {
    #[task(priority = 1, priority = 2)]
    fn foo(_: foo::Context) {}
};
