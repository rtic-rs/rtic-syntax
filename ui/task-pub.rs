#![no_main]

#[mock::app]
mod app {
    #[task]
    pub fn foo(_: foo::Context) {}
}
