#![no_main]

#[mock::app]
const APP: () = {
    #[task]
    fn foo() {}
};
