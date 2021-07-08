#![no_main]

#[mock::app]
mod app {
    #[task(capacity = 1, capacity = 2)]
    fn foo(_: foo::Context) {}
}
