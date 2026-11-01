//! Delay actual selection after original lease loss, then reclaim real roots.
use crate::facade::durability::{
    RelationalCheckpointAcknowledgementDenial, RelationalCheckpointArtifactPolicy,
};
use crate::tests::support::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn delayed_selection(empty: bool) {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    if !empty {
        create_entity(execution, &runtime, "original-selected-component");
    }
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let original_root = std::sync::Arc::downgrade(&basis.inner.root);
    let original_basis = std::sync::Arc::downgrade(&basis.inner);
    let original_schema = basis.inner.root.schema_authority().authority_digest();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);
    create_entity(execution, &runtime, "later-current-head");
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: runtime.durability.cold_source(),
            },
        )
        .unwrap();
    let observer = checkpoint.clone();
    runtime.release_component_basis(lease).unwrap();
    runtime.run_branch_root_reclamation_pass();
    assert!(
        original_root.upgrade().is_some(),
        "actual capture keeps the selected immutable root before filing"
    );
    let held = crate::branch::AdmittedRelationalBranchBasis {
        inner: original_basis
            .upgrade()
            .expect("the same actual observation survives original lease release"),
    };
    assert_eq!(
        held.inner.root.schema_authority().authority_digest(),
        original_schema
    );
    held.inner
        .root
        .demand_all_full(execution, &runtime.services.symbols)
        .unwrap();
    let read = runtime
        .snapshots()
        .snapshot_for_observation(&held.observation())
        .unwrap();
    runtime.snapshots().release_snapshot(&read).unwrap();
    drop(held);
    attached_test_medium(&runtime).file_checkpoint(checkpoint.clone());
    // Original selection is now actual. Interrupt the caller only after it
    // enters its one real execution scope and begins Native acknowledgement.
    let cancellation = CancellationSource::new();
    let backing =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let stopped_request = ExecutionRequest::serial(&backing);
    let mut entered = false;
    let _interrupted = stopped_request.run_native_with_checkpoint(|context| {
        entered = true;
        cancellation.cancel();
        let result = checkpoint.acknowledge_selected(&stopped_request, context);
        assert!(matches!(
            result,
            Err(RelationalCheckpointAcknowledgementDenial::Stopped(_))
        ));
        result
    });
    assert!(entered);
    drop(checkpoint);
    runtime.run_branch_root_reclamation_pass();
    assert!(
        original_basis.upgrade().is_some(),
        "partial acknowledgement and an old observer drop cannot retire the capture custody"
    );
    execution
        .run_native_with_checkpoint(|context| observer.acknowledge_selected(execution, context))
        .unwrap()
        .0
        .unwrap();
    assert!(original_basis.upgrade().is_none(), "successful original acknowledgement retires shared capture read custody even while observers remain");
    runtime.run_branch_root_reclamation_pass();
    assert!(
        original_root.upgrade().is_none(),
        "retired root is now reclaimable through the actual owner"
    );
    runtime.prepare_close().unwrap().finish();
    drop(observer);
}
#[test]
fn native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack() {
    delayed_selection(false);
}
#[test]
fn native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack() {
    delayed_selection(true);
}
#[test]
fn native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "abandoned-component");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let original = std::sync::Arc::downgrade(&basis.inner);
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);
    create_entity(execution, &runtime, "later-head");
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: runtime.durability.cold_source(),
            },
        )
        .unwrap();
    let observer = checkpoint.clone();
    runtime.release_component_basis(lease).unwrap();
    drop(checkpoint);
    runtime.run_branch_root_reclamation_pass();
    assert!(original.upgrade().is_some());
    drop(observer);
    assert!(
        original.upgrade().is_none(),
        "last actual unselected capture abandonment releases its original observation"
    );
    runtime.run_branch_root_reclamation_pass();
    runtime.prepare_close().unwrap().finish();
}
