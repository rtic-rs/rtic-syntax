#![no_main]

#[mock::app]
mod app {
    #[task(resources = [A])]
    fn foo(_: foo::Context) {}
}
