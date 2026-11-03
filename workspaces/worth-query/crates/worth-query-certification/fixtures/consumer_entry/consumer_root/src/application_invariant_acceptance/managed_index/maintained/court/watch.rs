//! Real public empty entrants, unchanged-candidate refinement and exact close.
use super::super::authoring::Edit;
use super::*;
mod coordinates;
mod race;
mod recovery;
mod refusal;

#[test]
fn predicate_watch_prefix_refreshes_full_parent_and_prunes_unrelated_probes() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::predicate_watch_prefix_refreshes_full_parent_and_prunes_unrelated_probes"
        ),
    );
}
fn run() {
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let closed = region.change();
    assert_eq!(
        closed.bytes_allocated, closed.bytes_deallocated,
        "watches, queued batches, Native metadata and runtime all close"
    );
}
fn journey() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install_watch(calls.clone(), rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let (initial, mut entrant) = request
        .query(find("A", "later"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert!(initial.rows().is_empty());
    let (initial, mut leaving) = request
        .query(find("A", "sh"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(values(initial.rows()), vec![("x".into(), "shared".into())]);
    let (_, mut other_scope) = request
        .query(find("B", "zz"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let mut disjoint = Vec::new();
    for index in 0..256 {
        disjoint.push(
            request
                .query(find("A", &format!("zz/{index:04}")))
                .watch_index::<prefix::PrefixIndex>()
                .unwrap()
                .1,
        );
    }
    let before = calls.predicate_regions.load(Ordering::SeqCst);
    let changed = request
        .advanced()
        .mutate(rename("sh", "later"))
        .expect_result_set(initial.result_set_observation().unwrap().clone())
        .idempotency(&0x9198_0100u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{changed:?}"
    );
    let visited = calls.predicate_regions.load(Ordering::SeqCst) - before;
    assert!(
        visited < 128,
        "one changed row must prune 256 unrelated probes; visited {visited}"
    );
    // A different parameter set is refused without acknowledging this watch.
    assert!(request
        .query(find("A", "wrong"))
        .next_watch(&mut entrant)
        .is_err());
    assert!(request
        .query(query::FindPrefixAlias {
            directory: "A".into(),
            prefix: "later".into()
        })
        .next_watch(&mut entrant)
        .is_err());
    let cancellation = WorthQueryCancellationSource::new();
    let interrupted = WorthQueryRequestScope::new(None, cancellation.token());
    let interrupted_in_read = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed_interruption = interrupted_in_read.clone();
    calls.matched_sources.store(1, Ordering::SeqCst);
    *calls.after_second_match.lock().unwrap() = Some(Box::new(move || {
        observed_interruption.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    assert!(
        world
            .runtime()
            .request(&principal, &interrupted)
            .on_branch(world.current_world())
            .query(find("A", "later"))
            .next_watch(&mut entrant)
            .is_err(),
        "interruption during the full read cannot ACK its queued notification"
    );
    assert!(
        interrupted_in_read.load(Ordering::SeqCst),
        "the queued refresh reached its real full-read cancellation observer"
    );
    let fresh = authentication::request_scope();
    let next = world
        .runtime()
        .request(&principal, &fresh)
        .on_branch(world.current_world());
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) = next
        .query(find("A", "later"))
        .next_watch(&mut entrant)
        .unwrap()
    else {
        panic!("empty watch must receive its entrant");
    };
    assert_eq!(
        values(update.read().rows()),
        vec![("x".into(), "later".into())]
    );
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) = next
        .query(find("A", "sh"))
        .next_watch(&mut leaving)
        .unwrap()
    else {
        panic!("before membership must wake a leaving watch");
    };
    assert!(update.read().rows().is_empty());
    assert!(matches!(
        next.query(find("B", "zz"))
            .next_watch(&mut other_scope)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert!(matches!(
        next.query(find("A", "later"))
            .next_watch(&mut entrant)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    for (index, watch) in disjoint.iter_mut().enumerate() {
        assert!(matches!(
            next.query(find("A", &format!("zz/{index:04}")))
                .next_watch(watch)
                .unwrap(),
            WorthQueryManagedIndexWatchOutcome::Pending
        ));
    }
    // enabled changes final membership while the provider's prefix candidates
    // remain identical. The ordinary full Parent must remove, then restore x.
    for (offset, enabled) in [false, true].into_iter().enumerate() {
        let changed = next
            .advanced()
            .mutate(Edit::SetEnabled {
                directory: "A".into(),
                item: "x".into(),
                enabled,
            })
            .idempotency(&(0x9198_0110u64 + offset as u64))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                changed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{changed:?}"
        );
        let WorthQueryManagedIndexWatchOutcome::Delivered(update) = next
            .query(find("A", "later"))
            .next_watch(&mut entrant)
            .unwrap()
        else {
            panic!("full Parent refinement changed");
        };
        assert_eq!(
            values(update.read().rows()),
            if enabled {
                vec![("x".into(), "later".into())]
            } else {
                Vec::new()
            }
        );
    }
    // Deletion and a fresh logical x must both route through the genuine Source.
    for (offset, edit) in [
        Edit::RetireItem {
            directory: "A".into(),
            item: "x".into(),
        },
        Edit::Item {
            directory: "A".into(),
            identity: "x".into(),
            name: "later".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let changed = next
            .advanced()
            .mutate(edit)
            .idempotency(&(0x9198_0120u64 + offset as u64))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                changed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{changed:?}"
        );
        let WorthQueryManagedIndexWatchOutcome::Delivered(update) = next
            .query(find("A", "later"))
            .next_watch(&mut entrant)
            .unwrap()
        else {
            panic!("deletion and a new entrant must each notify");
        };
        assert_eq!(
            values(update.read().rows()),
            if offset == 0 {
                Vec::new()
            } else {
                vec![("x".into(), "later".into())]
            }
        );
    }
    entrant.close().unwrap();
    leaving.close().unwrap();
    other_scope.close().unwrap();
    for watch in disjoint {
        watch.close().unwrap();
    }
    assert_eq!(
        values(initial.rows()),
        vec![("x".into(), "shared".into())],
        "the old full result stays immutable"
    );
    race::run(&world, &principal, &scope, &calls);
    refusal::run(&world, &principal, &scope, &calls);
    recovery::run(&world, &principal, &scope, &calls);
}
