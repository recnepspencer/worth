use super::*;

#[test]
fn runtime_public_support_gate_denies_deferred_and_unsupported_families_before_use() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.support-gate")
        .expect("task runtime should open a named workspace");

    let read = workspace
        .admit_public_api_family(WorthQueryRuntimeFacadeFamily::Read)
        .expect("supported read family should admit");
    assert_eq!(read.family(), WorthQueryRuntimeFacadeFamily::Read);
    assert_eq!(
        read.status(),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );

    for (family, expected_reason) in [
        (WorthQueryRuntimeFacadeFamily::Temporal, "support-gated"),
        (
            WorthQueryRuntimeFacadeFamily::AsyncResource,
            "support-gated",
        ),
        (
            WorthQueryRuntimeFacadeFamily::MixedCauseDelivery,
            "support-gated",
        ),
        (
            WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
            "Milestone 10",
        ),
        (
            WorthQueryRuntimeFacadeFamily::DurableArtifacts,
            "Milestone 11",
        ),
        (
            WorthQueryRuntimeFacadeFamily::Intent,
            "intent commit strategies",
        ),
    ] {
        let error = workspace
            .admit_public_api_family(family)
            .expect_err("unsupported or deferred public API family should fail closed");
        match error {
            WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
                assert_eq!(denial.family(), family);
                assert!(denial.reason().contains(expected_reason));
            }
            other => panic!("expected typed public support denial, got {other:?}"),
        }
    }
}

#[test]
fn runtime_public_api_naming_contract_prefers_workspace_surface_names() {
    let contract = WorthQueryRuntime::public_api_naming_contract();

    assert_eq!(contract.preferred_name_for("workspace"), Some("workspace"));
    assert_eq!(contract.preferred_name_for("insert"), Some("insert"));
    assert_eq!(contract.preferred_name_for("update"), Some("update"));
    assert_eq!(contract.preferred_name_for("delete"), Some("delete"));
    assert_eq!(contract.preferred_name_for("batch"), Some("batch"));
    assert_eq!(contract.preferred_name_for("inspect"), Some("inspect"));
    assert!(contract.rows().iter().any(|row| {
        row.concept() == "insert"
            && row
                .alternate_names()
                .iter()
                .any(|name| name.contains("WorthQueryWriteCommand::InsertAspects"))
    }));
    assert!(contract
        .rows()
        .iter()
        .all(|row| row.preferred_name() != "surface"));
}
