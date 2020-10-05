#![no_main]

#[mock::app]
mod app {
    #[task]
    fn foo(_: foo::Context) -> ! {
        loop {}
    }
}
