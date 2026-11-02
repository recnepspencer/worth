//! Public inverse box routing: exact occurrences and B/C backlog separation.
use super::*;

#[test]
fn predicate_watch_boxes_preserve_each_notification_with_current_full_receipts() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::predicate_watch_boxes_preserve_each_notification_with_current_full_receipts"
        ),
    );
}
fn run() {
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "spatial watches and each queued notification release all custody"
    );
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let prefix_calls = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(prefix_calls.clone()));
    let world = host::install_geometry(prefix_calls, prefix_rules, Configuration::new(calls));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let (original, mut staying) = request
        .query(find("A", 5))
        .watch_index::<provider::BoxIndex>()
        .unwrap();
    let (_, mut entrant) = request
        .query(find("A", 25))
        .watch_index::<provider::BoxIndex>()
        .unwrap();
    let (_, mut boundary) = request
        .query(find("A", 30))
        .watch_index::<provider::BoxIndex>()
        .unwrap();
    let (_, mut disjoint) = request
        .query(find("B", 50))
        .watch_index::<provider::BoxIndex>()
        .unwrap();
    let prefix_find = || super::super::super::query::FindPrefix {
        directory: "A".into(),
        prefix: "zz".into(),
    };
    let (_, mut prefix_watch) = request
        .query(prefix_find())
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let sibling = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(world.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    assert!(world
        .runtime()
        .request(&principal, &scope)
        .on_branch(sibling)
        .query(find("A", 5))
        .next_watch(&mut staying)
        .is_err());
    for (offset, x) in [20, 30].into_iter().enumerate() {
        commit(
            &world,
            &principal,
            &scope,
            Edit::Move {
                directory: "A".into(),
                vertex: "v1".into(),
                x,
                y: 0,
            },
            0x9198_0200 + offset as u64,
        );
    }
    let current = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let WorthQueryManagedIndexWatchOutcome::Delivered(first) = request
        .query(find("A", 5))
        .next_watch(&mut staying)
        .unwrap()
    else {
        panic!("first endpoint change must remain queued");
    };
    assert_ne!(
        first.notification().composite_commit(),
        &current,
        "B identifies the first actual publication"
    );
    let WorthQueryApplicationBasisSelectionIdentity::Product(read_at) =
        first.read().receipt().basis_identity().selection()
    else {
        panic!("whole Parent has a genuine World receipt");
    };
    assert_eq!(
        read_at.selected_commit(),
        &current,
        "C identifies the current completed read"
    );
    assert_eq!(
        values(first.read().rows()),
        expected("A", 30, find("A", 5).probe)
    );
    let WorthQueryManagedIndexWatchOutcome::Delivered(second) = request
        .query(find("A", 5))
        .next_watch(&mut staying)
        .unwrap()
    else {
        panic!("ACK of B must not ACK C");
    };
    assert_eq!(second.notification().composite_commit(), &current);
    assert_eq!(
        values(second.read().rows()),
        expected("A", 30, find("A", 5).probe)
    );
    assert!(matches!(
        request
            .query(find("A", 5))
            .next_watch(&mut staying)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    for (probe, watch) in [(25, &mut entrant), (30, &mut boundary)] {
        let WorthQueryManagedIndexWatchOutcome::Delivered(update) =
            request.query(find("A", probe)).next_watch(watch).unwrap()
        else {
            panic!("a closed boundary entrant must route");
        };
        assert_eq!(update.notification().composite_commit(), &current);
        assert_eq!(
            values(update.read().rows()),
            expected("A", 30, find("A", probe).probe)
        );
    }
    assert!(matches!(
        request
            .query(find("B", 50))
            .next_watch(&mut disjoint)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert!(matches!(
        request
            .query(prefix_find())
            .next_watch(&mut prefix_watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    staying.close().unwrap();
    entrant.close().unwrap();
    boundary.close().unwrap();
    disjoint.close().unwrap();
    prefix_watch.close().unwrap();
    assert_eq!(
        values(original.rows()),
        expected("A", 10, find("A", 5).probe)
    );
    world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(sibling).close(&phase).unwrap())
        .unwrap();
}
