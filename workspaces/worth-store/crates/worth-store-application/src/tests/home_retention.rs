//! Whole-Home frontier pins over a real Home's durable Store and over the
//! memory Home's volatile table (Store Milestone 4, Phase M4.1, `S4`).
use super::lifecycle::open;
use crate::facade::ApplicationHomeCloseOutcome;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_runtime_world::facade::{
    HomeFrontierComponents, HomeRootManifest, LeaseState, NestedDropPolicy, PhysicalRootPointer,
    PinId, PinScope, PinSpec, RetentionPortDenial, RuntimeWorldRetentionPort,
};

fn spec(scope: u64, parent: Option<PinId>) -> PinSpec {
    PinSpec {
        scope: PinScope::new(scope),
        parent_pin: parent,
        nested_drop_policy: NestedDropPolicy::RetainUntilChildrenDrain,
    }
}

fn closed(home: crate::facade::ReadmittedApplicationHome, execution: &ExecutionRequest<'_, '_>) {
    match home.close(execution) {
        ApplicationHomeCloseOutcome::Closed { .. } => {}
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => panic!("close: {reason:?}"),
    }
}

#[test]
fn frontier_pin_binds_every_component_and_survives_commits_and_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let home = open(directory.path(), &execution);
    super::native_schema::commit(home.relational().runtime(), &execution, "before-pin");
    let retention = home.frontier_retention();
    let lease = retention.capture_and_pin_frontier(spec(1, None)).unwrap();
    let manifest = lease.manifest().clone();
    let generation = manifest.frontier_sequence();
    for pointer in [
        manifest.native_root(),
        manifest.world_root(),
        manifest.signal_root(),
        manifest.product_map_root(),
    ] {
        assert_eq!(pointer.root_generation(), generation);
    }
    assert!(
        manifest.native_root().position() > 0,
        "the native commit is pinned"
    );
    assert_eq!(manifest.home_incarnation(), home.incarnation());
    assert_eq!(
        manifest.installed_program_digest(),
        *home.binding().selected().revision().bytes()
    );
    assert_eq!(
        HomeRootManifest::decode(&manifest.encode()),
        Some(manifest.clone())
    );

    // HEAD advances; the pinned frontier does not.
    super::native_schema::commit(home.relational().runtime(), &execution, "after-pin");
    assert_eq!(lease.read_pinned_manifest().unwrap(), manifest);
    let child = retention
        .capture_and_pin_frontier(spec(2, Some(lease.pin_id())))
        .unwrap();
    assert!(child.manifest().frontier_sequence() > generation);
    assert!(child.manifest().native_root().position() > manifest.native_root().position());
    let (parent, _) = lease.into_catalog_pin();
    let (child, child_manifest) = child.into_catalog_pin();
    drop(retention);
    closed(home, &execution);

    let home = open(directory.path(), &execution);
    let retention = home.frontier_retention();
    assert_eq!(retention.pinned_manifest(parent).unwrap(), manifest);
    assert_eq!(retention.pinned_manifest(child).unwrap(), child_manifest);
    assert_eq!(retention.lease_state(parent), Some(LeaseState::Active));
    // The parent cannot release its roots while the child overlay lives.
    assert_eq!(
        retention.expire_lease(parent).unwrap().state(),
        LeaseState::Expiring
    );
    assert_eq!(retention.pinned_manifest(parent).unwrap(), manifest);
    assert_eq!(
        retention.expire_lease(child).unwrap().state(),
        LeaseState::Expired
    );
    assert_eq!(retention.lease_state(parent), Some(LeaseState::Expired));
    assert_eq!(
        retention.pinned_manifest(parent),
        Err(RetentionPortDenial::PinNotFound)
    );
    drop(retention);
    closed(home, &execution);
}

#[test]
fn memory_home_retention_runs_the_same_nested_lease_law() {
    let frontier = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1));
    let source = std::sync::Arc::clone(&frontier);
    let retention = crate::home_retention::volatile::VolatileHomeRetention::new(move || {
        let sequence = source.load(std::sync::atomic::Ordering::SeqCst);
        let at = |position| PhysicalRootPointer::from_retention_owner(sequence, position);
        HomeRootManifest::seal(
            worth_foundational::facade::HomeIncarnation::initial(),
            sequence,
            HomeFrontierComponents {
                native_root: at(1),
                world_root: at(2),
                signal_root: at(3),
                product_map_root: at(4),
            },
            [7; 32],
        )
    });
    let parent = retention.capture_and_pin_frontier(spec(1, None)).unwrap();
    frontier.store(2, std::sync::atomic::Ordering::SeqCst);
    let child = retention
        .capture_and_pin_frontier(spec(2, Some(parent.pin_id())))
        .unwrap();
    frontier.store(3, std::sync::atomic::Ordering::SeqCst);
    let grandchild = retention
        .capture_and_pin_frontier(spec(3, Some(child.pin_id())))
        .unwrap();
    let parent_pin = parent.pin_id();
    assert_eq!(
        retention.expire_lease(parent_pin).unwrap().state(),
        LeaseState::Expiring
    );
    assert_eq!(
        parent.read_pinned_manifest(),
        Err(RetentionPortDenial::LeaseExpired)
    );
    assert!(grandchild.read_pinned_manifest().is_ok());
    drop(child);
    for sequence in 1..=3 {
        assert!(retention.protects_frontier(sequence), "frontier {sequence}");
    }
    // Dropping the grandchild drains the chain bottom-up.
    drop(grandchild);
    assert_eq!(retention.lease_state(parent_pin), Some(LeaseState::Expired));
    for sequence in 1..=3 {
        assert!(
            !retention.protects_frontier(sequence),
            "frontier {sequence}"
        );
    }
    drop(parent);
}
