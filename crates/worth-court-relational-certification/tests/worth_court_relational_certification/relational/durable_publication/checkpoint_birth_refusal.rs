//! Standalone byte restoration carries no original birth authority.
use super::*;
use worth_relational::facade::durability::{
    RelationalCheckpointArtifactPolicy, RelationalNativeCheckpointCaptureDenial as Denial,
};
#[test]
fn complete_refuses_recovered_empty_without_original_birth_authority() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&setup);
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    let ordinary = source
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let receiver_medium = TestDurableMedium::new();
    let mut receiver = durable_runtime(&receiver_medium);
    receiver
        .durability_recovery()
        .restore_native_checkpoint(&execution, &ordinary)
        .unwrap();
    assert!(matches!(
        receiver
            .complete_checkpoint_service()
            .prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded),
        Err(Denial::OriginalAuthorityUnavailable)
    ));
}
