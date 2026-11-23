//! The last actual unselected capture alias releases its original custody.
use crate::facade::durability::RelationalCheckpointArtifactPolicy;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

#[test]
fn native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
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
