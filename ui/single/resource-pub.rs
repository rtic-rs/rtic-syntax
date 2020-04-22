#![no_main]

#[mock::app]
mod app {
    struct Resources {
        #[init(0)]
        pub x: u32,
    }
}
