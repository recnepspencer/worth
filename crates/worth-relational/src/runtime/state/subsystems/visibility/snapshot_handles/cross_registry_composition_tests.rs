use crate::facade::history::BranchId;
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotId;
use crate::tests::support::*;

/// The three identities a cross-registry observer must be able to tell apart.
#[derive(Clone, Copy, Debug)]
struct CrossRegistryIdentities {
    active: SnapshotId,
    published: SnapshotId,
    absent: SnapshotId,
}

/// A runtime holding one live handle in each registry, plus one identity that
/// was minted and never admitted anywhere.
///
/// Both witnesses stay resident for the whole proof, so an observer really does
/// have to consult both registries; a world with an empty registry would answer
/// from the first lookup and prove nothing.
fn with_cross_registry_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    proof: impl FnOnce(&RelationalRuntime, CrossRegistryIdentities),
) {
    let runtime = runtime_with_test_schema();
    let published = create_entity_outcome(execution, &runtime, "cross-registry-witness");
    let active = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let identities = CrossRegistryIdentities {
        active: active.snapshot_id(),
        published: published.snapshot.snapshot_id(),
        absent: runtime
            .visibility
            .allocate_snapshot_id()
            .expect("the world mints one identity it never admits"),
    };
    assert!(
        runtime.visibility.active_snapshot_count() > 0,
        "the world must hold a live active handle for the whole proof"
    );
    assert!(
        runtime.visibility.published_snapshot_handle_count() > 0,
        "the world must hold a live published handle for the whole proof"
    );

    proof(&runtime, identities);

    drop(active);
    drop(published);
}

/// The cross-registry answer is a union taken one registry at a time, active
/// first and published second. Losing either arm of that composition, or
/// admitting an identity that was never installed, shows up here.
///
/// This test proves the composition, not the lock discipline. Nothing in the
/// discipline is left to a runtime check: the guard types are private to their
/// own registry files and cannot be named from the pair facade, so a paired
/// cross-registry hold is not writable in the first place. See the lock
/// discipline note on `SnapshotHandles`.
#[test]
fn a_known_snapshot_is_found_in_whichever_registry_holds_it() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    with_cross_registry_world(execution, |runtime, identities| {
        assert!(runtime.visibility.is_known_snapshot(identities.active));
        assert!(runtime.visibility.is_known_snapshot(identities.published));
        assert!(!runtime.visibility.is_known_snapshot(identities.absent));
    });
}
