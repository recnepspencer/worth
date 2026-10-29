//! Actual box adapter anchors for branch incarnation and retained historical basis.
use super::super::{host, lifecycle, mutation, prefix};
use super::{geometry, query, BoxIndex, Calls, Coordinates};
use crate::application_invariant_acceptance::authentication;
use lifecycle::{authenticate, run};
use std::sync::Arc;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

fn find() -> query::FindBoxes {
    geometry::input("boxes", Coordinates::new(2, 2, 2, 2))
}
fn refusal(
    error: WorthQueryApplicationRequestQueryDenial,
    expected: WorthQueryManagedIndexRefusal,
) {
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = error else {
        panic!("actual managed box refusal: {error:?}")
    };
    assert!(
        matches!(cause.cause(), WorthQueryManagedIndexDenialCause::Refused(actual) if actual == expected),
        "{cause:?}"
    );
}
#[test]
fn registered_box_rejects_same_runtime_foreign_branch_and_retained_stale_basis() {
    run(branch_and_basis);
}
fn branch_and_basis() {
    let calls = Arc::new(Calls::default());
    let world = host::install_boxes(Arc::new(prefix::Calls::default()), Arc::clone(&calls));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let branch = world
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
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let other = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let index = request.query(find()).open_index::<BoxIndex>().unwrap();
    request.query(find()).rebuild_index(&index).unwrap();
    let source = request
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    let expected = geometry::expected(source.rows(), find().probe);
    assert_eq!(request.query(find()).execute().unwrap().rows(), expected);
    let before = calls.totals();
    refusal(
        other.query(find()).rebuild_index(&index).unwrap_err(),
        WorthQueryManagedIndexRefusal::ForeignAssociation,
    );
    geometry::not_ready(
        other
            .query(find())
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(
        calls.totals(),
        before,
        "same-runtime foreign branch refuses before box callbacks"
    );
    let observation = request.retain_current_read().unwrap();
    let pinned = request.at(&observation);
    let old_source = pinned
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    // A genuine publication on this branch changes the basis. Whole-branch
    // invalidation remains conservative even though this edit changes a name.
    // The existing rename binding requires its genuine managed prefix Source.
    // Keep that installed contract intact while publishing the new box basis.
    let prefix_input = || super::super::query::FindPrefix {
        directory: "catalog".into(),
        prefix: "al".into(),
    };
    let prefix_index = request
        .query(prefix_input())
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(prefix_input())
        .rebuild_index(&prefix_index)
        .unwrap();
    let rename_source = request
        .query(prefix_input())
        .execute()
        .unwrap()
        .into_result_set_observation()
        .unwrap();
    let changed = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "catalog".into(),
            prefix: "al".into(),
            item: "item-a".into(),
            replacement: "alba".into(),
        })
        .expect_result_set(rename_source)
        .idempotency(&0x919d004u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    prefix_index.close();
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        observation.selected_commit()
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    let before = calls.totals();
    refusal(
        pinned
            .query(find())
            .execute()
            .err()
            .expect("managed read must refuse"),
        WorthQueryManagedIndexRefusal::UnsupportedMode,
    );
    refusal(
        pinned.query(find()).rebuild_index(&index).unwrap_err(),
        WorthQueryManagedIndexRefusal::UnsupportedMode,
    );
    geometry::not_ready(
        request
            .query(find())
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(
        calls.totals(),
        before,
        "stale basis cannot invoke the actual box provider"
    );
    let historical = pinned
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        historical.rows(),
        old_source.rows(),
        "ordinary historical Source retains its meaning"
    );
    request.query(find()).rebuild_index(&index).unwrap();
    let current_source = request
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    let result = request.query(find()).execute().unwrap();
    assert_eq!(
        result.rows(),
        geometry::expected(current_source.rows(), find().probe)
    );
    assert_eq!(result.rows(), expected);
    index.close();
    drop((
        result,
        current_source,
        historical,
        old_source,
        pinned,
        observation,
        changed,
        source,
        other,
    ));
    world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(branch).close(&phase).unwrap())
        .unwrap();
}
