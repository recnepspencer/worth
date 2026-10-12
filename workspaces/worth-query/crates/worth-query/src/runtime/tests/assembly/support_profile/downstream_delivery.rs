use super::super::super::support::*;

#[test]
fn runtime_public_downstream_delivery_contract_freezes_runtime_backed_and_durable_resume_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_runtime(request_execution);
    let contract = runtime.public_downstream_delivery_contract();

    assert_eq!(
        contract.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(
        contract.runtime_resume_support_posture(),
        WorthQueryLowerRuntimeSupportPosture::Admitted
    );
    assert_eq!(
        contract.durable_resume_support_posture(),
        WorthQueryLowerRuntimeSupportPosture::Deferred
    );
    assert!(contract.runtime_backed_resume_supported());
    assert!(contract.durable_resume_deferred());
    assert!(!contract.runtime_resume_support_for_reporting().is_empty());
    assert!(!contract.durable_resume_support_for_reporting().is_empty());
    assert!(!contract.contract_for_reporting().is_empty());
}

#[test]
fn runtime_public_support_matrix_exposes_downstream_delivery_contract_row() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.downstream-delivery-support")
        .expect("task runtime should open a named workspace");
    let matrix = workspace.public_support_matrix();
    let row = matrix
        .row("downstream-delivery-contract")
        .expect("downstream delivery contract row should stay explicit");

    assert_eq!(
        row.status(),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert_eq!(
        row.teaching_posture(),
        WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
    );
    assert!(row.parallel_api_forbidden());
    assert!(!row.admission_fail_closed());
    assert_eq!(row.owner_milestone(), "Milestone 9.4");
    assert_eq!(
        row.support_contract_digest(),
        Some(
            workspace
                .public_downstream_delivery_contract()
                .contract_for_reporting()
        )
    );
}
