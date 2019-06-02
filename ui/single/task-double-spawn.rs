#![no_main]

#[mock::app]
const APP: () = {
    #[task(spawn = [bar], spawn = [baz])]
    fn foo(_: foo::Context) {}
};
