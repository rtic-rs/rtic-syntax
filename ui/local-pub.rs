#![no_main]

#[mock::app]
mod app {
    #[local]
    struct Local {
        pub x: u32,
    }
}
