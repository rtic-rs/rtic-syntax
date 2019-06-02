#![no_main]

#[mock::app]
const APP: () = {
    #[task(resources = [A])]
    fn foo(_: foo::Context) {}
};
