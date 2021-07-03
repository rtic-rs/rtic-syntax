#![no_main]

#[mock::app]
mod app {
    #[task(shared = [A], shared = [B])]
    fn foo(_: foo::Context) {}
}
