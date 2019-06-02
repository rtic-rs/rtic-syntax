#![no_main]

#[mock::app]
const APP: () = {
    #[task(resources = [A], resources = [B])]
    fn foo(_: foo::Context) {}
};
