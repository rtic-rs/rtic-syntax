#![no_main]

#[mock::app]
const APP: () = {
    struct Resources {
        #[shared]
        x: u32,
    }
};
