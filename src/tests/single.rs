use crate::{
    analyze::{Ownership, Spawnee},
    Settings,
};
use quote::quote;

#[test]
fn unused_task() {
    // this shouldn't crash the analysis pass
    crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {}

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task]
                fn foo(_: foo::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();
}

#[test]
fn shared_resource_owned() {
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [x])]
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
fn shared_resource_coowned() {
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [x])]
                fn foo(_: foo::Context) {}

                #[task(shared = [x])]
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
fn shared_resource_contended() {
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [x])]
                fn foo(_: foo::Context) {}

                #[task(priority = 2, shared = [x])]
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
fn no_send_late_shared_resources_idle() {
    // late resources owned by `idle` don't need to be `Send`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[idle(shared = [x])]
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
                #[shared]
                struct Shared {}

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

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
fn send_shared_resource() {
    // shared resources used by tasks must be `Send`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    a: X,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [a])]
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
fn send_local_resource() {
    // local resources used by tasks from the Local struct must be `Send`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {}

                #[local]
                struct Local {
                    a: X,
                }

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(local = [a])]
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
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [x])]
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
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [x])]
                fn foo(_: foo::Context) {}

                #[task(shared = [x])]
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
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [&x])]
                fn foo(_: foo::Context) {}

                #[task(priority = 2, shared = [&x])]
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
fn not_sync2() {
    // `static` resources shared between same priority tasks do not need to be `Sync`
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [&x])]
                fn foo(_: foo::Context) {}

                #[task(shared = [&x])]
                fn bar(_: bar::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    assert!(analysis.sync_types.is_empty());
}

#[test]
fn not_sync3() {
    // `static` resources between different priority tasks do not need to be `Sync`, protected by
    // the mutex
    let (_app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {}

                #[task(shared = [x])]
                fn foo(_: foo::Context) {}

                #[task(priority = 2, shared = [x])]
                fn bar(_: bar::Context) {}
            }
        ),
        Settings::default(),
    )
    .unwrap();

    assert!(analysis.sync_types.is_empty());
}

#[test]
fn late_resources() {
    // Check so that late resources gets initialized
    let (app, _analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[shared]
                struct Shared {
                    x: i32,
                }

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
                    ..
                }
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let late = &app.shared_resources;
    assert_eq!(late.len(), 1);
}

#[test]
fn actors() {
    let (app, analysis) = crate::parse2(
        quote!(),
        quote!(
            mod app {
                #[actors]
                struct Actors {
                    #[priority = 2]
                    #[subscribe(MsgA)]
                    #[subscribe(MsgB, capacity = 2)]
                    a: (),

                    #[subscribe(MsgA)]
                    #[init(1)]
                    b: u32,

                    #[priority = 1]
                    #[subscribe(MsgB, capacity = 3)]
                    c: (),
                }

                #[shared]
                struct Shared {}

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics, Actors) {
                    ..
                }
            }
        ),
        Settings::default(),
    )
    .unwrap();

    let actors = &app.actors;
    assert_eq!(actors.len(), 3);

    {
        let actor = &app.actors[0];
        assert_eq!(2, actor.priority);
        assert_eq!(1, actor.subscriptions[0].capacity);
        assert_eq!(2, actor.subscriptions[1].capacity);
        assert_eq!(None, actor.init);
    }

    {
        let actor = &app.actors[1];

        assert_eq!(1, actor.priority);
        assert_eq!(1, actor.subscriptions[0].capacity);
        assert_eq!(
            Some("1"),
            actor
                .init
                .as_ref()
                .map(|expr| quote!(#expr).to_string())
                .as_deref()
        );
    }

    {
        let actor = &app.actors[2];

        assert_eq!(1, actor.priority);
        assert_eq!(3, actor.subscriptions[0].capacity);
        assert_eq!(None, actor.init);
    }

    let expected = [
        (
            /* priority: */ 1,
            /* capacity */ 4,
            [("b", 0), ("c", 0)],
        ),
        (
            /* priority: */ 2,
            /* capacity */ 3,
            [("a", 0), ("a", 1)],
        ),
    ];
    for (actual, expected) in analysis.channels.iter().zip(expected) {
        assert_eq!(*actual.0, expected.0);
        assert_eq!(actual.1.capacity, expected.1);
        for (spawnee, expected) in actual.1.spawnees.iter().zip(expected.2) {
            match spawnee {
                Spawnee::Task { .. } => panic!(),
                Spawnee::Actor {
                    name,
                    subscription_index,
                } => {
                    assert_eq!(expected.0, name.to_string());
                    assert_eq!(expected.1, *subscription_index);
                }
            }
        }
    }
}
