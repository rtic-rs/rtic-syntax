#![no_main]

#[mock::app]
mod app {
    #[task(priority = 256)]
    fn foo(_: foo::Context) {}
}
