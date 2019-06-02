#![no_main]

#[mock::app]
const APP: () = {
    #[idle(resources = [A], resources = [B])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
};
