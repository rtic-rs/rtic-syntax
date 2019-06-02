#![no_main]

#[mock::app]
const APP: () = {
    pub static X: u32 = 0;
};
