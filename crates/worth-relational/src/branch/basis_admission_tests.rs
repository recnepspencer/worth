use super::*;
use crate::config::data::runtime_config::RelationalRuntimeConfig;
use crate::history::data::BranchId;
use crate::runtime::RelationalRuntime;

fn runtime() -> RelationalRuntime {
    RelationalRuntime::new(RelationalRuntimeConfig::default())
        .expect("valid native runtime configuration")
}

#[test]
fn observation_and_clones_share_one_repeatable_empty_basis() {
    let runtime = runtime();
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    let cloned = basis.clone();

    assert_eq!(descriptor.root_identity(), 0);
    assert_eq!(observation.selected_root_identity(), 0);
    assert_eq!(observation.descriptor(), cloned.descriptor());
    assert_eq!(
        basis.retention_reason(),
        crate::history::retention::RelationalBasisRetentionReason::Observation
    );
    assert_eq!(
        basis.admission_identity(),
        cloned.admission_identity(),
        "clones preserve the owner-issued admission identity"
    );
}

#[test]
fn reset_retention_owner_reports_unavailable_readmission_and_terminal_release() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity(execution, &runtime, "owner-reset-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);

    runtime.set_retention_capacity_for_test(128, 128);

    assert!(matches!(
        runtime.readmit_retained_branch_basis(&descriptor, &lease),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
    let receipt = runtime.release_component_basis(lease).unwrap();
    assert_eq!(
        receipt.outcome(),
        crate::history::retention::RelationalBranchRetentionTerminalOutcome::OwnerUnavailable
    );
}

#[test]
fn archived_and_unsupported_descriptors_deny_distinctly() {
    let runtime = runtime();
    let identity = runtime.main_branch_identity();
    let (descriptor, _) = runtime.observe_branch(&identity).unwrap();

    let mut payload = serde_json::to_value(&descriptor).unwrap();
    payload["posture"] = serde_json::json!(RelationalBranchBasisPosture::Archived);
    let archived: RelationalBranchBasisDescriptor =
        serde_json::from_value(payload.clone()).unwrap();
    assert!(matches!(
        runtime.readmit_branch_basis(&archived),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));

    payload["posture"] = serde_json::json!(descriptor.posture());
    payload["descriptor_version"] = serde_json::json!(99);
    let unsupported: RelationalBranchBasisDescriptor = serde_json::from_value(payload).unwrap();
    assert!(matches!(
        runtime.readmit_branch_basis(&unsupported),
        Err(RelationalBranchBasisDenial::UnsupportedDescriptorVersion { actual: 99, .. })
    ));

    assert_eq!(runtime.history.branch_count(), 1);
    assert_eq!(identity.branch_id(), &BranchId("main".to_owned()));
}
