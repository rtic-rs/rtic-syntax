#![no_main]

#[mock::app]
mod app {
    #[task(priority = 0)]
    fn foo(_: foo::Context) {}
}
