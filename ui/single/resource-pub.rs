#![no_main]

#[mock::app]
const APP: () = {
    struct Resources {
        #[init(0)]
        pub x: u32,
    }
};
