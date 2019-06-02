#![no_main]

#[mock::app]
const APP: () = {
    #[task(priority = 0)]
    fn foo(_: foo::Context) {}
};
