#![no_main]

#[mock::app]
mod app {
    #[shared]
    struct Shared {
        pub x: u32,
    }
}
