#![no_main]

#[mock::app]
mod app {
    #[task(priority = 1, priority = 2)]
    fn foo(_: foo::Context) {}
}
