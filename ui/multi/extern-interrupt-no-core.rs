#![no_main]

#[mock::app(cores = 2, parse_cores, parse_extern_interrupt)]
const APP: () = {
    extern "C" {
        fn foo();
    }
};
