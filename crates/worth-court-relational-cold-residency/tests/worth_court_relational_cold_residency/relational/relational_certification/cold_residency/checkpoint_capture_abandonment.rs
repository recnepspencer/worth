//! The last actual unselected capture alias releases its original custody.
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::RelationalCheckpointArtifactPolicy;
use worth_testkit_relational::{
    durable_publication::{create_entity, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    create_entity(execution, &runtime, "abandoned-component");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let original = basis.custody_witness();
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
                source: None,
            },
        )
        .unwrap();
    let observer = checkpoint.clone();
    runtime.release_component_basis(lease).unwrap();
    drop(checkpoint);
    runtime.run_branch_root_reclamation_pass();
    assert!(original.basis_retained());
    drop(observer);
    assert!(
        !original.basis_retained(),
        "last actual unselected capture abandonment releases its original observation"
    );
    runtime.run_branch_root_reclamation_pass();
    runtime.prepare_close().unwrap().finish();
}
