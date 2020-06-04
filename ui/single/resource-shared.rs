#![no_main]

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        #[shared]
        x: u32,
    }
}
