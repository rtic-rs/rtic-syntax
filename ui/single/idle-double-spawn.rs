#![no_main]

#[mock::app]
const APP: () = {
    #[idle(spawn = [foo], spawn = [bar])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
};
