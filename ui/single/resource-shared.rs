#![no_main]

#[mock::app]
mod app {
    struct Resources {
        #[shared]
        x: u32,
    }
}
