#![no_main]

#[mock::app]
mod app {
    #[task]
    unsafe fn foo(_: foo::Context) {}
}
