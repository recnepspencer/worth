use bank_domain::estate::{
    CapabilityGrantStatus, EmergencyAccessReason, EmergencyAccessStatus, EstateCapabilityOperation,
    EstateCapabilityPurpose, EstateMoment, EstateWorkflowStage, MandatoryReviewKind,
    MandatoryReviewStatus, RestrictedBankField,
};
use bank_domain::model::Money;
use bank_domain::queries::EstateGovernanceQuery;
use bank_domain::reads::EstateGovernanceContext;
use worth_query_host::facade::publication::domain_computation::{
    WorthQueryPublishedApplicationDisclosurePosture,
    WorthQueryPublishedApplicationQueryOmissionPosture, WorthQueryPublishedApplicationResult,
};

use super::fixture::{
    capability_world, governance_projection_world, request_scope, GrantSpec, ACCOUNT, APPROVER,
    BRANCH, CLOSED_ACCESS, COMPLETED_REVIEW, DECEASED, DELEGATED_GRANT, DISBURSEMENT_GRANT,
    EMERGENCY_BOUND_GRANT, ESTATE, GRANT, INSTITUTION, REQUESTED_ACCESS, REQUESTED_REVIEW,
    REVIEWER, SPECIALIST,
};
use crate::{queries, BankApplicationQueryDenial, BankReadControls};

type GovernanceResult =
    WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>;

#[test]
fn public_estate_governance_query_consumes_the_exact_administration_capability() {
    let fixture = governance_projection_world("governed-estate-query");
    let principal = fixture.authenticate();
    let result = fixture
        .runtime
        .query(queries::estate_governance_context(ESTATE))
        .as_principal(&principal)
        .controls(controls())
        .execute()
        .expect("the exact administration capability should govern the estate query");

    assert_governance_context(&result);
    assert_governance_receipt(&result);
}

fn assert_governance_context(result: &GovernanceResult) {
    assert_eq!(result.rows().len(), 1);
    let context = &result.rows()[0];
    assert_governance_root(context);
    assert_projected_capabilities(context);
}

fn assert_governance_root(context: &EstateGovernanceContext) {
    assert_eq!(context.estate(), ESTATE);
    assert_eq!(context.stage(), EstateWorkflowStage::Administration);
    assert!(context.beneficiaries().is_empty());
    assert_eq!(context.assignments().len(), 3);
    assert_eq!(context.capabilities().len(), 4);
}

fn assert_projected_capabilities(context: &EstateGovernanceContext) {
    let capability = context
        .capabilities()
        .iter()
        .find(|capability| capability.id() == GRANT)
        .unwrap();
    assert_eq!(capability.id(), GRANT);
    assert_eq!(
        capability.operation(),
        EstateCapabilityOperation::ViewRestrictedEstate
    );
    assert_eq!(
        capability.purpose(),
        EstateCapabilityPurpose::EstateAdministration
    );
    assert_eq!(
        capability.workflow_stage(),
        EstateWorkflowStage::Administration
    );
    assert_eq!(capability.status(), CapabilityGrantStatus::Active);
    assert_eq!(capability.grantee(), SPECIALIST);
    assert_eq!(capability.grantor(), DECEASED);
    assert_eq!(capability.account(), None);
    assert_eq!(capability.estate(), ESTATE);
    assert_eq!(capability.institution(), INSTITUTION);
    assert_eq!(capability.branch(), BRANCH);
    assert_eq!(
        capability.field(),
        Some(RestrictedBankField::GovernanceMetadata)
    );
    assert_eq!(capability.amount_ceiling(), None);
    assert_eq!(capability.parent(), None);
    assert_eq!(capability.delegation().remaining(), 1);
    assert!(capability.emergencies().is_empty());

    assert_delegated_capability(context, capability);
    assert_disbursement_capability(context);
    assert_emergency_bound_capability(context);
}

fn assert_delegated_capability(
    context: &EstateGovernanceContext,
    parent: &bank_domain::reads::EstateCapabilityContext,
) {
    let delegated = find_capability(context, DELEGATED_GRANT);
    assert_eq!(delegated.parent(), Some(GRANT));
    assert_eq!(delegated.grantor(), DECEASED);
    assert_eq!(delegated.grantee(), APPROVER);
    assert_eq!(delegated.status(), CapabilityGrantStatus::Active);
    assert_eq!(delegated.account(), parent.account());
    assert_eq!(delegated.estate(), parent.estate());
    assert_eq!(delegated.institution(), parent.institution());
    assert_eq!(delegated.branch(), parent.branch());
    assert_eq!(delegated.operation(), parent.operation());
    assert_eq!(delegated.purpose(), parent.purpose());
    assert_eq!(delegated.field(), parent.field());
    assert_eq!(delegated.amount_ceiling(), parent.amount_ceiling());
    assert_eq!(delegated.validity(), parent.validity());
    assert_eq!(delegated.workflow_stage(), parent.workflow_stage());
    assert_eq!(delegated.delegation().remaining(), 0);
    assert!(delegated.emergencies().is_empty());
}

fn assert_disbursement_capability(context: &EstateGovernanceContext) {
    let disbursement = find_capability(context, DISBURSEMENT_GRANT);
    assert_eq!(disbursement.grantor(), DECEASED);
    assert_eq!(disbursement.grantee(), REVIEWER);
    assert_eq!(disbursement.status(), CapabilityGrantStatus::Active);
    assert_eq!(disbursement.account(), Some(ACCOUNT));
    assert_eq!(disbursement.estate(), ESTATE);
    assert_eq!(disbursement.institution(), INSTITUTION);
    assert_eq!(disbursement.branch(), BRANCH);
    assert_eq!(
        disbursement.operation(),
        EstateCapabilityOperation::DisburseEstate
    );
    assert_eq!(
        disbursement.purpose(),
        EstateCapabilityPurpose::EstateDisbursement
    );
    assert_eq!(
        disbursement.amount_ceiling(),
        Some(Money::from_minor(50_000).unwrap())
    );
    assert_eq!(disbursement.field(), None);
    assert_eq!(
        disbursement.valid_from(),
        EstateMoment::from_epoch_seconds(0)
    );
    assert_eq!(
        disbursement.valid_through(),
        EstateMoment::from_epoch_seconds(u64::MAX)
    );
    assert_eq!(
        disbursement.workflow_stage(),
        EstateWorkflowStage::Administration
    );
    assert_eq!(disbursement.delegation().remaining(), 0);
    assert_eq!(disbursement.parent(), None);
    assert!(disbursement.emergencies().is_empty());
}

fn assert_emergency_bound_capability(context: &EstateGovernanceContext) {
    let emergency_bound = find_capability(context, EMERGENCY_BOUND_GRANT);
    assert_eq!(emergency_bound.grantor(), DECEASED);
    assert_eq!(emergency_bound.grantee(), SPECIALIST);
    assert_eq!(emergency_bound.status(), CapabilityGrantStatus::Active);
    assert_eq!(emergency_bound.account(), None);
    assert_eq!(emergency_bound.estate(), ESTATE);
    assert_eq!(emergency_bound.institution(), INSTITUTION);
    assert_eq!(emergency_bound.branch(), BRANCH);
    assert_eq!(
        emergency_bound.operation(),
        EstateCapabilityOperation::ViewRestrictedEstate
    );
    assert_eq!(
        emergency_bound.purpose(),
        EstateCapabilityPurpose::EmergencyProtection
    );
    assert_eq!(
        emergency_bound.field(),
        Some(RestrictedBankField::AccountDetails)
    );
    assert_eq!(emergency_bound.amount_ceiling(), None);
    assert_eq!(emergency_bound.parent(), None);
    assert_eq!(emergency_bound.delegation().remaining(), 0);
    assert_eq!(
        emergency_bound.workflow_stage(),
        EstateWorkflowStage::Administration
    );
    assert_eq!(emergency_bound.emergencies().len(), 2);
    assert_requested_emergency(emergency_bound);
    assert_closed_emergency(emergency_bound);
}

fn find_capability(
    context: &EstateGovernanceContext,
    id: bank_domain::estate::CapabilityGrantId,
) -> &bank_domain::reads::EstateCapabilityContext {
    context
        .capabilities()
        .iter()
        .find(|capability| capability.id() == id)
        .unwrap()
}

fn assert_requested_emergency(capability: &bank_domain::reads::EstateCapabilityContext) {
    let access = capability
        .emergencies()
        .iter()
        .find(|access| access.id() == REQUESTED_ACCESS)
        .unwrap();
    assert_eq!(access.grant(), EMERGENCY_BOUND_GRANT);
    assert_eq!(access.review(), REQUESTED_REVIEW);
    assert_eq!(access.requester(), SPECIALIST);
    assert_eq!(access.approver(), None);
    assert_eq!(access.reviewer(), None);
    assert_eq!(access.reason(), EmergencyAccessReason::PreventImmediateLoss);
    assert_eq!(access.status(), EmergencyAccessStatus::Requested);
    assert_eq!(access.issued_at(), EstateMoment::from_epoch_seconds(100));
    assert_eq!(access.expires_at(), EstateMoment::from_epoch_seconds(200));
    let review = access.mandatory_review();
    assert_eq!(review.estate, ESTATE);
    assert_eq!(review.kind, MandatoryReviewKind::EmergencyAccess);
    assert_eq!(review.reviewer, None);
    assert_eq!(review.status, MandatoryReviewStatus::Required);
}

fn assert_closed_emergency(capability: &bank_domain::reads::EstateCapabilityContext) {
    let access = capability
        .emergencies()
        .iter()
        .find(|access| access.id() == CLOSED_ACCESS)
        .unwrap();
    assert_eq!(access.grant(), EMERGENCY_BOUND_GRANT);
    assert_eq!(access.review(), COMPLETED_REVIEW);
    assert_eq!(access.requester(), SPECIALIST);
    assert_eq!(access.approver(), Some(APPROVER));
    assert_eq!(access.reviewer(), Some(REVIEWER));
    assert_eq!(access.reason(), EmergencyAccessReason::MeetLegalDeadline);
    assert_eq!(access.status(), EmergencyAccessStatus::Revoked);
    assert_eq!(access.issued_at(), EstateMoment::from_epoch_seconds(300));
    assert_eq!(access.expires_at(), EstateMoment::from_epoch_seconds(400));
    let review = access.mandatory_review();
    assert_eq!(review.estate, ESTATE);
    assert_eq!(review.kind, MandatoryReviewKind::EmergencyAccess);
    assert_eq!(review.reviewer, Some(REVIEWER));
    assert_eq!(review.status, MandatoryReviewStatus::Completed);
}

fn assert_governance_receipt(result: &GovernanceResult) {
    let disclosure = result.receipt().disclosure();
    assert_eq!(
        disclosure.posture(),
        WorthQueryPublishedApplicationDisclosurePosture::Governed
    );
    assert_eq!(disclosure.disclosure_decision_count(), 50);
    assert_eq!(disclosure.omitted_value_count(), 0);
    assert!(disclosure.authorization_decision_fact_count() > 0);
    let inspection = result.receipt().inspect();
    assert_eq!(
        inspection.omission_posture(),
        WorthQueryPublishedApplicationQueryOmissionPosture::NoOmission
    );
    assert_eq!(inspection.result_count(), 1);
    assert!(inspection.terminal_resources_released());
}

#[test]
fn public_estate_governance_query_rejects_one_axis_purpose_and_field_substitution() {
    let mut wrong_purpose = GrantSpec::governance_view();
    wrong_purpose.purpose = EstateCapabilityPurpose::IdentityVerification;
    assert_governance_denied("governance-wrong-purpose", wrong_purpose);

    let mut wrong_field = GrantSpec::governance_view();
    wrong_field.field = Some(RestrictedBankField::CustomerIdentity);
    assert_governance_denied("governance-wrong-field", wrong_field);
}

fn assert_governance_denied(scenario: &str, grant: GrantSpec) {
    let fixture = capability_world(scenario, grant, EstateWorkflowStage::Administration);
    let principal = fixture.authenticate();
    let denial = fixture
        .runtime
        .query(queries::estate_governance_context(ESTATE))
        .as_principal(&principal)
        .controls(controls())
        .execute()
        .err()
        .expect("a substituted grant dimension must deny the governed query");

    let BankApplicationQueryDenial::CapabilityAdmission(denial) = denial else {
        panic!("the substituted grant must fail at capability admission: {denial:#?}")
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::CapabilityAuthorizationMissing
    );
}

fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 1).unwrap()
}
