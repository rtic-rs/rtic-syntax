#![no_main]

#[mock::app(parse_binds)]
mod app {
    #[task(binds = SysTick, only_same_priority_spawn_please_fix_me)]
    fn foo(_: foo::Context) {}
}
