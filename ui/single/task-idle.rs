#![no_main]

#[mock::app]
const APP: () = {
    #[task]
    fn idle(_: idle::Context) {}
};
