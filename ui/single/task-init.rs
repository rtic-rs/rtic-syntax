#![no_main]

#[mock::app]
mod app {
    #[init]
    fn foo(_: foo::Context) {}

    // name collides with `#[idle]` function
    #[task]
    fn foo(_: foo::Context) {}
}
