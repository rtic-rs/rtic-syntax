#![no_main]

#[mock::app(cores = 2, parse_cores)]
const APP: () = {
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
};
