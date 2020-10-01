#![no_main]

#[mock::app]
const APP: () = {
    #[init]
    fn foo(_: foo::Context) -> foo::LateResources {}

    // name collides with `#[idle]` function
    #[task]
    fn foo(_: foo::Context) {}
};
