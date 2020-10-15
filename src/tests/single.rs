use quote::quote;

use crate::{analyze::Ownership, Settings};

#[test]
fn unused_resource() {
    // this shouldn't crash the analysis pass
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                struct Resources {
                    #[init(0)]
                    x: i32,
                }
            }
        ),
        Settings::default(),
    )
    .unwrap();

    // `x` shouldn't be listed in `locations`
    assert!(analysis.locations.is_empty());
}

#[test]
fn unused_task() {
    // this shouldn't crash the analysis pass
    crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[task]
                fn foo(_: foo::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();
}

#[test]
fn resource_owned() {
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    #[init(0)]
                    x: i32,
                }

                #[task(resources = [x])]
                fn foo(_: foo::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let (res, ownership) = analysis.ownerships.iter().next().unwrap();
    assert_eq!(res.to_string(), "x");
    assert_eq!(*ownership, Ownership::Owned { priority: 1 });
}

#[test]
fn resource_coowned() {
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    #[init(0)]
                    x: i32,
                }

                #[task(resources = [x])]
                fn foo(_: foo::Context) {}

                #[task(resources = [x])]
                fn bar(_: bar::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let (res, ownership) = analysis.ownerships.iter().next().unwrap();
    assert_eq!(res.to_string(), "x");
    assert_eq!(*ownership, Ownership::CoOwned { priority: 1 });
}

#[test]
fn resource_contended() {
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    #[init(0)]
                    x: i32,
                }

                #[task(resources = [x])]
                fn foo(_: foo::Context) {}

                #[task(priority = 2, resources = [x])]
                fn bar(_: bar::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let (res, ownership) = analysis.ownerships.iter().next().unwrap();
    assert_eq!(res.to_string(), "x");
    assert_eq!(*ownership, Ownership::Contended { ceiling: 2 });
}

#[test]
fn no_send_late_resources_idle() {
    // late resources owned by `idle` don't need to be `Send`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    x: i32,
                }

                #[init]
                fn init(_: init::Context) -> init::LateResources {
                    ..
                }

                #[idle(resources = [x])]
                fn idle(_: idle::Context) -> ! {
                    loop {}
                }
            }
        ),
        Settings::default(),
    )
    .unwrap();

    assert!(analysis.send_types.is_empty());
}

#[test]
fn send_spawn() {
    // message passing between different priority tasks needs a `Send` bound
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[task(priority = 2)]
                fn foo(_: foo::Context) {}

                #[task]
                fn bar(_: bar::Context, _: X) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let ty = analysis.send_types.iter().next().unwrap();
    assert_eq!(quote!(#ty).to_string(), "X");
}

#[test]
fn send_late_resource() {
    // late resources used by tasks must be `Send`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    a: X,
                }

                #[init]
                fn init(_: init::Context) -> init::LateResources {
                    ..
                }

                #[task(resources = [a])]
                fn foo(_: foo::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let ty = analysis.send_types.iter().next().unwrap();
    assert_eq!(quote!(#ty).to_string(), "X");
}

#[test]
fn send_shared_with_init() {
    // resources shared with `init` must be `Send`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    #[init(0)]
                    x: i32,
                }

                #[init(resources = [x])]
                fn init(_: init::Context) -> init::LateResources {}

                #[task(resources = [x])]
                fn foo(_: foo::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let ty = analysis.send_types.iter().next().unwrap();
    assert_eq!(quote!(#ty).to_string(), "i32");
}

#[test]
fn not_sync() {
    // `static` resources shared between same priority tasks don't need a `Sync` bound
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    #[init(0)]
                    x: i32,
                }

                #[task(resources = [x])]
                fn foo(_: foo::Context) {}

                #[task(resources = [x])]
                fn bar(_: bar::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    assert!(analysis.sync_types.is_empty());
}

#[test]
fn sync() {
    // `static` resources shared between different priority tasks need to be `Sync`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    #[init(0)]
                    x: i32,
                }

                #[task(resources = [&x])]
                fn foo(_: foo::Context) {}

                #[task(priority = 2, resources = [&x])]
                fn bar(_: bar::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let ty = analysis.sync_types.iter().next().unwrap();
    assert_eq!(quote!(#ty).to_string(), "i32");
}

#[test]
fn late_resources() {
    // Check so that late resources gets initialized
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[resources]
                struct Resources {
                    x: i32,
                }

                #[init]
                fn init(_: init::Context) -> init::LateResources {
                    ..
                }
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let late = &analysis.late_resources;
    assert_eq!(late.len(), 1);
}
