#![no_main]

#[mock::app]
const APP: () = {
    #[idle]
    fn idle(_: idle::Context) -> ! {
        #[shared]
        static mut X: [u8; 128] = [0; 128];

        loop {}
    }
};
