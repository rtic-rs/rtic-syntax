#![no_main]

#[mock::app]
mod app {
    #[task(local = [A], local = [B])]
    fn foo(_: foo::Context) {}
}
