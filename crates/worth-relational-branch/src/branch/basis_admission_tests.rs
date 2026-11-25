//! Direct Branch fixtures exercise installed cells, observation, and retention.
use super::basis::{RelationalBranchBasisDescriptor, RelationalBranchBasisPosture};
use super::basis_denial::RelationalBranchBasisDenial;
use super::basis_fixture::observation_inputs;
pub(crate) use super::basis_fixture::{
    installed_history_for_test as installed_history,
    observed_empty_branch_for_test as observed_empty_branch,
};
use crate::history::data::coordinates::BranchId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::RelationalSchemaRegistry;

#[test]
fn observation_and_clones_share_one_repeatable_empty_basis() {
    let history = installed_history(1);
    let schema = RelationalSchemaRegistry::new();
    let instrumentation = RuntimeInstrumentation::new();
    let inputs = observation_inputs(&history, &schema, &instrumentation);
    let identity = inputs.branch_identity(&history.main_branch).unwrap();
    let (descriptor, basis) = inputs.observe(&identity, || None).unwrap();
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
    let mut history = installed_history(1);
    let schema = RelationalSchemaRegistry::new();
    let instrumentation = RuntimeInstrumentation::new();
    let inputs = observation_inputs(&history, &schema, &instrumentation);
    let identity = inputs.branch_identity(&history.main_branch).unwrap();
    let (descriptor, basis) = inputs.observe(&identity, || None).unwrap();
    let retained_root = std::sync::Arc::downgrade(&basis.inner.root);
    let lease =
        super::basis_retention::retain_component_basis(&history, 1, &instrumentation, &basis)
            .unwrap();
    drop(basis);

    history.reset_retention_owner_with_test_limits_for_test(128, 128);
    assert!(retained_root.upgrade().is_some());

    assert!(matches!(
        super::basis_retention::readmit_retained_branch_basis(
            &history,
            1,
            &instrumentation,
            &descriptor,
            &lease
        ),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
    let receipt = super::basis_retention::release_component_basis(&history, 1, lease).unwrap();
    assert_eq!(
        receipt.outcome(),
        crate::history::retention::RelationalBranchRetentionTerminalOutcome::OwnerUnavailable
    );
}

#[test]
fn archived_and_unsupported_descriptors_deny_distinctly() {
    let history = installed_history(1);
    let schema = RelationalSchemaRegistry::new();
    let instrumentation = RuntimeInstrumentation::new();
    let inputs = observation_inputs(&history, &schema, &instrumentation);
    let identity = inputs.branch_identity(&history.main_branch).unwrap();
    let (descriptor, _) = inputs.observe(&identity, || None).unwrap();

    let mut payload = serde_json::to_value(&descriptor).unwrap();
    payload["posture"] = serde_json::json!(RelationalBranchBasisPosture::Archived);
    let archived: RelationalBranchBasisDescriptor =
        serde_json::from_value(payload.clone()).unwrap();
    assert!(matches!(
        inputs.readmit_branch_basis(&archived),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));

    payload["posture"] = serde_json::json!(descriptor.posture());
    payload["descriptor_version"] = serde_json::json!(99);
    let unsupported: RelationalBranchBasisDescriptor = serde_json::from_value(payload).unwrap();
    assert!(matches!(
        inputs.readmit_branch_basis(&unsupported),
        Err(RelationalBranchBasisDenial::UnsupportedDescriptorVersion { actual: 99, .. })
    ));

    assert_eq!(history.branch_count(), 1);
    assert_eq!(identity.branch_id(), &BranchId("main".to_owned()));
}
