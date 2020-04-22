#![no_main]

#[mock::app]
mod app {
    #[task(resources = [A], resources = [B])]
    fn foo(_: foo::Context) {}
}
