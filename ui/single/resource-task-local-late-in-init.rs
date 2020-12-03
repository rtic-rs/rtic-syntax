#![no_main]

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        #[task_local]
        x: u32,
    }

    #[init(resources = [x])]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {}

    #[task(resources = [x])]
    fn foo(c: foo::Context) {
        c.resources.x += 1;
    }
}
