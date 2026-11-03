//! Genuine checkpoint claims survive a busy owner and stopped original work.
use super::fixture::real_fixture;
use crate::durability::WorldCheckpointOrigin;
use crate::lifecycle::RuntimeWorldCloseDenial;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn checkpoint_origin_preserves_original_claims_across_busy_and_cancelled_retirement() {
    let fixture = real_fixture(4, 4);
    let head = fixture.owner.issue_product_head(&fixture.basis).unwrap();
    let pins = head.fork_history(&fixture.basis).unwrap();
    let keys = pins.retention_keys();
    let mut origin = WorldCheckpointOrigin::new(&fixture.basis, pins);
    drop(head);
    let live = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&live);
    let before = fixture.owner.cost_snapshot();
    let native_before = fixture.relational_runtime.branch_basis_cost_counters();
    let guard = fixture.owner.state.lock().unwrap();
    assert_eq!(
        origin.advance_retirement(&request, &fixture.relational_runtime),
        Err(RuntimeWorldCloseDenial::Busy)
    );
    drop(guard);
    assert_eq!(fixture.owner.cost_snapshot(), before);
    assert_eq!(
        fixture.relational_runtime.branch_basis_cost_counters(),
        native_before
    );
    let cancel = CancellationSource::new();
    cancel.cancel();
    let stopped = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    assert!(matches!(
        origin.advance_retirement(
            &ExecutionRequest::serial(&stopped),
            &fixture.relational_runtime
        ),
        Err(RuntimeWorldCloseDenial::Execution(_))
    ));
    assert_eq!(fixture.owner.active_component_obligation_count(), 2);
    assert_eq!(fixture.owner.cost_snapshot(), before);
    while !origin
        .advance_retirement(&request, &fixture.relational_runtime)
        .unwrap()
    {}
    let retired = origin.into_retired().unwrap();
    assert_eq!(retired.retention_keys().cloned().collect::<Vec<_>>(), keys);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(
        fixture.owner.cost_snapshot().owner_drop_releases(),
        before.owner_drop_releases()
    );
    assert_eq!(
        fixture
            .relational_runtime
            .branch_basis_cost_counters()
            .external_retention_drop_releases,
        native_before.external_retention_drop_releases
    );
    drop(retired);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
}
