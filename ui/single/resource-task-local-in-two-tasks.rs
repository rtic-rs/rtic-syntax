#![no_main]

#[mock::app]
mod app {
    #[resources]
    struct Resources {
        #[task_local]
        x: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        init::LateResources {}
    }

    #[task(
                resources = [x],
            )]
    fn foo(c: foo::Context) {
        c.resources.x += 1;
    }

    #[task(
                resources = [x],
            )]
    fn bar(c: bar::Context) {
        c.resources.x += 1;
    }
}
