use bank_domain::{
    estate::{
        CapabilityGrantStatus, EstateAction, EstateCapabilityPurpose, EstateWorkflowStage,
        RestrictedBankField,
    },
    schema::{ViewEstateAdministrationCapability, ViewRestrictedEstateOperation},
};
use worth_query_host::facade::{
    declaration::application_schema::ApplicationScalarValueBinding,
    primary_graph::WorthQueryOperationAuthorizationDenialKind,
};

use super::fixture::{capability_world, request_scope, GrantSpec, ESTATE};

#[test]
fn active_bank_grant_mints_only_a_current_move_only_access_proof() {
    let fixture = capability_world(
        "active-capability",
        GrantSpec::view(),
        EstateWorkflowStage::Administration,
    );
    let principal = fixture.authenticate();
    let application = fixture.runtime.application_runtime();
    let capability = application
        .installed_schema()
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    let request = request_scope();
    let selected = fixture.runtime.select_current_product().unwrap();
    let access = selected
        .admit_capability_access(principal.query(), &capability, view_action(), &request)
        .unwrap();

    assert_eq!(access.operation(), "ViewRestrictedEstateOperation");
    assert_eq!(access.authorization_decision_fact_count(), 2);
    assert_eq!(
        access.projected_request().field_value(),
        Some(
            &bank_domain::schema::RestrictedBankFieldBinding::encode(
                &RestrictedBankField::CustomerIdentity,
            )
            .unwrap(),
        )
    );
    assert_eq!(access.capability_time_sample().semantic_byte_width(), 9);
    assert!(access.relational_counters().paths_evaluated > 0);
    assert!(access.signal_dependency_count() > 0);
    assert_zero_canonical_work(access.admission_canonical_work());
    assert_eq!(capability.lookup_evidence().basis_preparations(), 0);
    assert_eq!(capability.lookup_evidence().digest_derivations(), 0);
    assert_eq!(
        capability.lookup_evidence().digest_text_materializations(),
        0
    );
}

#[test]
fn revoked_expired_future_and_workflow_drifted_grants_fail_current_admission() {
    let mut revoked = GrantSpec::view();
    revoked.status = CapabilityGrantStatus::Revoked;
    assert_eq!(
        view_denial("revoked", revoked, EstateWorkflowStage::Administration),
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
    );

    let mut expired = GrantSpec::view();
    expired.not_after = 1;
    assert_eq!(
        view_denial("expired", expired, EstateWorkflowStage::Administration),
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
    );

    let mut future = GrantSpec::view();
    future.not_before = u64::MAX;
    assert_eq!(
        view_denial("future", future, EstateWorkflowStage::Administration),
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
    );

    let mut workflow = GrantSpec::view();
    workflow.workflow = EstateWorkflowStage::AuthorityReview;
    assert_eq!(
        view_denial(
            "workflow-drift",
            workflow,
            EstateWorkflowStage::Administration
        ),
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
    );
}

fn view_denial(
    scenario: &str,
    spec: GrantSpec,
    stage: EstateWorkflowStage,
) -> WorthQueryOperationAuthorizationDenialKind {
    let fixture = capability_world(scenario, spec, stage);
    let principal = fixture.authenticate();
    let application = fixture.runtime.application_runtime();
    let capability = application
        .installed_schema()
        .capability(
            ViewEstateAdministrationCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
        )
        .unwrap();
    fixture
        .runtime
        .select_current_product()
        .unwrap()
        .admit_capability_access(
            principal.query(),
            &capability,
            view_action(),
            &request_scope(),
        )
        .err()
        .expect("the hostile grant should deny")
        .kind()
}

pub(super) fn view_action() -> EstateAction {
    EstateAction::ViewRestrictedEstate {
        estate: ESTATE,
        field: RestrictedBankField::CustomerIdentity,
        purpose: EstateCapabilityPurpose::EstateAdministration,
    }
}

fn assert_zero_canonical_work(
    work: worth_query_host::facade::domain::WorthQueryCanonicalWorkEvidence,
) {
    assert_eq!(work.basis_preparations(), 0);
    assert_eq!(work.digest_derivations(), 0);
    assert_eq!(work.canonical_entries(), 0);
    assert_eq!(work.canonical_encoded_bytes(), 0);
    assert_eq!(work.canonical_material_allocation_bytes(), 0);
    assert_eq!(work.sha256_input_bytes(), 0);
    assert_eq!(work.sha256_compression_blocks(), 0);
    assert_eq!(work.digest_text_materializations(), 0);
}
