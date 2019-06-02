#![no_main]

#[mock::app]
const APP: () = {
    #[task(capacity = 1, capacity = 2)]
    fn foo(_: foo::Context) {}
};
