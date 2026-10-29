//! Genuine branch, retained occurrence and reclaimed full-generation controls.
use super::super::{directory, mutation};
use super::*;
use worth_query_host::facade::{
    product::WorthQueryProductBranch, runtime::ExecutionAllocationPolicy,
};

#[test]
fn registered_prefix_rejects_same_runtime_foreign_branch_and_retained_stale_basis() {
    run(branch_and_basis);
}
fn branch_and_basis() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let branch = fork(&world);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let other = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let index = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("al")).rebuild_index(&index).unwrap();
    let before = calls.totals();
    refusal(
        other.query(find("al")).rebuild_index(&index).unwrap_err(),
        WorthQueryManagedIndexRefusal::ForeignAssociation,
    );
    not_ready(
        other
            .query(find("al"))
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(
        calls.totals(),
        before,
        "same runtime and equal scalar inputs cannot cross branch incarnation"
    );
    let sibling_branch_index = other
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    other
        .query(find("al"))
        .rebuild_index(&sibling_branch_index)
        .unwrap();
    let observation = request.retain_current_read().unwrap();
    let pinned = request.at(&observation);
    let old_source = pinned
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    let rename_source = request
        .query(find("al"))
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
        .idempotency(&0x919d001u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        observation.selected_commit()
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    assert_eq!(
        sibling_branch_index.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready,
        "only publication on the actual owning branch invalidates a root"
    );
    let before = calls.totals();
    refusal(
        pinned
            .query(find("al"))
            .execute()
            .err()
            .expect("managed read must refuse"),
        WorthQueryManagedIndexRefusal::UnsupportedMode,
    );
    refusal(
        pinned.query(find("al")).rebuild_index(&index).unwrap_err(),
        WorthQueryManagedIndexRefusal::UnsupportedMode,
    );
    not_ready(
        request
            .query(find("al"))
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(
        calls.totals(),
        before,
        "stale retained basis cannot reinterpret a cached root or reconstruct"
    );
    let still_old = pinned
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        still_old.rows(),
        old_source.rows(),
        "historical ordinary reads keep their real meaning"
    );
    request.query(find("al")).rebuild_index(&index).unwrap();
    let source = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    let actual = request.query(find("al")).execute().unwrap();
    assert_eq!(actual.rows(), journey::expected(source.rows(), "al"));
    assert!(actual
        .rows()
        .iter()
        .any(|row| row.identity == "item-a" && row.name == "alba"));
    let unchanged = other.query(find("al")).execute().unwrap();
    assert_eq!(unchanged.rows(), journey::expected(old_source.rows(), "al"));
    index.close();
    sibling_branch_index.close();
    drop((
        unchanged,
        actual,
        source,
        still_old,
        old_source,
        pinned,
        observation,
        changed,
        other,
    ));
    world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(branch).close(&phase).unwrap())
        .unwrap();
}
fn fork(world: &host::Host) -> WorthQueryProductBranch {
    world
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
        .unwrap()
}

#[test]
fn registered_prefix_rejects_genuine_reclaimed_scope_generation_replacement() {
    run(scope_generation);
}
fn scope_generation() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let empty = || query::FindPrefix {
        directory: "empty".into(),
        prefix: "".into(),
    };
    let read = || directory::Read {
        directory: "empty".into(),
    };
    let original = request.query(read()).execute().unwrap().rows()[0].coordinate;
    let index = request
        .query(empty())
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(empty()).rebuild_index(&index).unwrap();
    assert!(request.query(empty()).execute().unwrap().rows().is_empty());
    let retired = request
        .advanced()
        .mutate(directory::Replace {
            edit: directory::Edit::Retire,
        })
        .without_source()
        .idempotency(&0x919d002u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        retired,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    // Drop the real publication/Source observation owners before Native maintenance.
    // No arena writes, generation mutation or simulated slot-reuse doors are used.
    drop(retired);
    let reclaimed = world
        .runtime()
        .with_application_advancement(&scope, |phase| {
            world.runtime().try_reclaim_retired_storage(&phase).unwrap()
        })
        .unwrap();
    assert_eq!(
        reclaimed.entity_reclaimed, 1,
        "the actual retired scope is reclaimed by its installed owner"
    );
    let created = request
        .advanced()
        .mutate(directory::Replace {
            edit: directory::Edit::Create,
        })
        .without_source()
        .idempotency(&0x919d003u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        created,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let replacement = request.query(read()).execute().unwrap().rows()[0].coordinate;
    assert_eq!(
        (replacement.0, replacement.1),
        (original.0, original.1),
        "actual Native partition and local slot reuse"
    );
    assert!(
        replacement.2 > original.2,
        "Native issued a genuinely newer generation"
    );
    let before = calls.totals();
    refusal(
        request.query(empty()).rebuild_index(&index).unwrap_err(),
        WorthQueryManagedIndexRefusal::ForeignAssociation,
    );
    not_ready(
        request
            .query(empty())
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(
        calls.totals(),
        before,
        "equal directory identity and raw slot cannot launder the old generation"
    );
    let fresh = request
        .query(empty())
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
    request.query(empty()).rebuild_index(&fresh).unwrap();
    let source = request
        .query(query::ReadSource {
            directory: "empty".into(),
        })
        .execute()
        .unwrap();
    let actual = request.query(empty()).execute().unwrap();
    assert_eq!(actual.rows(), journey::expected(source.rows(), ""));
    assert!(
        actual.rows().is_empty(),
        "the replacement scope has no inherited membership"
    );
    index.close();
    assert_eq!(
        fresh.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready,
        "closing the old generation's registration cannot dispose the new one"
    );
    fresh.close();
}
