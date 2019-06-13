#![no_main]

#[mock::app]
const APP: () = {
    #[idle]
    fn foo(_: foo::Context) -> ! {
        loop {}
    }

    // name collides with `#[idle]` function
    #[task]
    fn foo(_: foo::Context) {}
};
