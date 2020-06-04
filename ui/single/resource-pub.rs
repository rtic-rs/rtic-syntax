#![no_main]

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        #[init(0)]
        pub x: u32,
    }
}
