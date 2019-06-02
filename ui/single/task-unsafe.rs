#![no_main]

#[mock::app]
const APP: () = {
    #[task]
    unsafe fn foo(_: foo::Context) {}
};
