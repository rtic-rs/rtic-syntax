#![no_main]

#[mock::app]
const APP: () = {
    #[task]
    pub fn foo(_: foo::Context) {}
};
