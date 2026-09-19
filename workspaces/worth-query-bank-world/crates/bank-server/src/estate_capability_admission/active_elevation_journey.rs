//! Successful Bank emergency-use lifecycle and authoritative readback evidence.

use std::time::Duration;

use bank_domain::{
    estate::{
        BankDisclosure, EmergencyAccessId, EmergencyAccessReason, EmergencyAccessStatus,
        EstateAction, EstateWorkflowStage, MandatoryReviewId, MandatoryReviewStatus,
        RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
    queries::EstateGovernanceQuery,
    reads::EstateGovernanceContext,
    schema::AccountStatus,
};
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationResult;
use worth_query_host::facade::publication::domain_computation::{
    WorthQueryPublishedApplicationDisclosurePosture,
    WorthQueryPublishedApplicationQueryOmissionPosture,
};

use super::{
    fixture::{
        emergency_request_world, request_scope, CapabilityFixture, GrantSpec, ACCOUNT, ESTATE,
        GRANT, REVIEWER, SPECIALIST,
    },
    lifecycle_journey::{
        approve_elevation, request_elevation, ElevationApprovalSpec, ElevationRequestSpec,
    },
};
use crate::{
    queries, BankAuthenticatedPrincipal, BankEstateElevationCloseOutcome,
    BankEstateMandatoryReview, BankEstateMandatoryReviewOutcome, BankReadControls,
};

type GovernanceResult =
    WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>;

#[derive(Clone, Copy)]
struct EmergencyJourneyIdentity {
    access: EmergencyAccessId,
    review: MandatoryReviewId,
}

#[test]
fn approved_emergency_discloses_account_details_and_terminal_state_reads_back() {
    let fixture = emergency_request_world(
        "estate-emergency-active-use",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let reviewer = fixture.authenticate_reviewer();
    let identity = EmergencyJourneyIdentity {
        access: EmergencyAccessId::new(361).unwrap(),
        review: MandatoryReviewId::new(362).unwrap(),
    };
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 361,
            review: 362,
            idempotency: 91,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let approved = approve_elevation(
        &fixture,
        &approver,
        requested,
        ElevationApprovalSpec {
            access: 361,
            idempotency: 93,
        },
    );

    assert_approved_account_disclosure(&fixture, &requester, identity.access, &approved);
    let mandatory = close_elevation(&fixture, &approver, approved, identity.access);
    assert_closed_readback(&fixture, &requester, &approver, identity);
    complete_review(&fixture, &reviewer, mandatory, identity);
    assert_completed_readback(&fixture, &requester, identity);
}

/// Q8.26-C6: a commit retains a pre-image exactly when its installed contract
/// declares one — and the negative twin holds in the same journey.
///
/// `ApproveEmergencyAccess` declares `RecordedInverse` over
/// `EmergencyAccessStatusField`, so its commit must carry the demanded slice.
/// `RequestEmergencyAccess` is a create lane declared `not_correctable`: there
/// is no prior truth, so it must carry nothing. Asserting only the positive
/// would pass against a runtime that retained indiscriminately, which would
/// make every receipt claim an inverse it cannot honour.
#[test]
fn retention_follows_the_declared_mechanism_on_both_lifecycle_commits() {
    let fixture = emergency_request_world(
        "estate-emergency-retention-by-mechanism",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 371,
            review: 372,
            idempotency: 95,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let approved = approve_elevation(
        &fixture,
        &approver,
        requested,
        ElevationApprovalSpec {
            access: 371,
            idempotency: 97,
        },
    );

    assert!(
        approved.approval_retained_preimage_present(),
        "ApproveEmergencyAccess declares RecordedInverse/ExactPriorTruth, so its \
         commit must carry the pre-image its installed contract demands"
    );
    assert!(
        !approved.request_retained_preimage_present(),
        "RequestEmergencyAccess is a not_correctable create lane — it has no \
         prior truth, so its commit must retain nothing"
    );
    assert!(
        approved.approval_prior_status_is_requested(),
        "approval must retain the exact pre-commit Requested status"
    );
    let demanded = approved
        .approval_retention_work()
        .expect("approval commit carries retention work");
    assert!(demanded.validated_intents_examined() > 0);
    assert!(demanded.mutation_targets_materialized() > 0);
    assert!(demanded.decision_facts_examined() > 0);
    assert!(demanded.candidates_materialized() > 0);
    assert_eq!(demanded.demanded_loci_examined(), 1);

    let ordinary = approved
        .request_retention_work()
        .expect("request commit carries zero-valued work evidence");
    assert_eq!(ordinary.validated_intents_examined(), 0);
    assert_eq!(ordinary.mutation_targets_materialized(), 0);
    assert_eq!(ordinary.decision_facts_examined(), 0);
    assert_eq!(ordinary.candidates_materialized(), 0);
    assert_eq!(ordinary.demanded_loci_examined(), 0);
}

fn assert_approved_account_disclosure(
    fixture: &CapabilityFixture,
    requester: &BankAuthenticatedPrincipal,
    access: EmergencyAccessId,
    approved: &crate::BankApprovedEstateElevation,
) {
    let published = fixture
        .runtime
        .query(queries::estate_emergency_account_details(ESTATE, access))
        .as_principal(requester)
        .controls(controls())
        .execute_with_approved_elevation(approved)
        .expect("the exact approved field should reach the public Bank query");
    assert_eq!(published.rows().len(), 1);
    let BankDisclosure::Disclosed(account) = published.rows()[0].account() else {
        panic!("the exact approved account details must be disclosed");
    };
    assert_eq!(account.id(), ACCOUNT);
    assert_eq!(account.display_name().as_str(), "Estate Operating");
    assert_eq!(account.status(), AccountStatus::Frozen);
    let publication = published.receipt().inspect();
    assert_eq!(
        publication.omission_posture(),
        WorthQueryPublishedApplicationQueryOmissionPosture::NoOmission
    );
    assert_eq!(publication.result_count(), published.rows().len());
    assert!(publication.terminal_resources_released());
    let disclosure = published.receipt().disclosure();
    assert_eq!(
        disclosure.posture(),
        WorthQueryPublishedApplicationDisclosurePosture::Governed
    );
    assert_eq!(disclosure.disclosure_decision_count(), 4);
    assert_eq!(disclosure.disclosed_value_count(), 4);
    assert_eq!(disclosure.omitted_value_count(), 0);
    assert!(disclosure.authorization_decision_fact_count() > 0);
}

fn close_elevation(
    fixture: &CapabilityFixture,
    approver: &BankAuthenticatedPrincipal,
    approved: crate::BankApprovedEstateElevation,
    access: EmergencyAccessId,
) -> BankEstateMandatoryReview {
    let close = fixture
        .runtime
        .revoke_estate_emergency_access_with_key(
            approver,
            approved,
            EstateAction::RevokeEmergencyAccess {
                estate: ESTATE,
                access,
            },
            &bank_idempotency(95),
            &request_scope(),
        )
        .expect("the used elevation should remain closable through its exact command");
    let BankEstateElevationCloseOutcome::Closed(mandatory) = close else {
        panic!("the close must commit before readback: {close:?}");
    };
    mandatory
}

fn assert_closed_readback(
    fixture: &CapabilityFixture,
    requester: &BankAuthenticatedPrincipal,
    approver: &BankAuthenticatedPrincipal,
    identity: EmergencyJourneyIdentity,
) {
    let closed = governance_readback(fixture, requester);
    let emergency = emergency(&closed, identity.access);
    assert_eq!(emergency.status(), EmergencyAccessStatus::Revoked);
    assert_eq!(emergency.grant(), GRANT);
    assert_eq!(emergency.requester(), SPECIALIST);
    assert_eq!(
        emergency.reason(),
        EmergencyAccessReason::PreventImmediateLoss
    );
    assert_eq!(emergency.approver(), Some(approver.principal_id()));
    assert_eq!(emergency.reviewer(), None);
    assert_eq!(emergency.mandatory_review().id, identity.review);
    assert_eq!(
        emergency.mandatory_review().status,
        MandatoryReviewStatus::Required
    );
    assert_eq!(emergency.mandatory_review().reviewer, None);
}

fn complete_review(
    fixture: &CapabilityFixture,
    reviewer: &BankAuthenticatedPrincipal,
    mandatory: BankEstateMandatoryReview,
    identity: EmergencyJourneyIdentity,
) {
    let outcome = fixture
        .runtime
        .complete_estate_mandatory_review_with_key(
            reviewer,
            mandatory,
            EstateAction::CompleteMandatoryReview {
                estate: ESTATE,
                access: identity.access,
                review: identity.review,
            },
            &bank_idempotency(97),
            &request_scope(),
        )
        .expect("the exact mandatory review should commit after readback");
    let BankEstateMandatoryReviewOutcome::Reviewed(_) = outcome else {
        panic!("the exact review must be fresh: {outcome:?}");
    };
}

fn assert_completed_readback(
    fixture: &CapabilityFixture,
    requester: &BankAuthenticatedPrincipal,
    identity: EmergencyJourneyIdentity,
) {
    let completed = governance_readback(fixture, requester);
    let emergency = emergency(&completed, identity.access);
    assert_eq!(emergency.grant(), GRANT);
    assert_eq!(emergency.requester(), SPECIALIST);
    assert_eq!(emergency.review(), identity.review);
    assert_eq!(emergency.reviewer(), Some(REVIEWER));
    assert_eq!(
        emergency.mandatory_review().status,
        MandatoryReviewStatus::Completed
    );
    assert_eq!(emergency.mandatory_review().reviewer, Some(REVIEWER));
}

fn governance_readback(
    fixture: &CapabilityFixture,
    observer: &BankAuthenticatedPrincipal,
) -> GovernanceResult {
    fixture
        .runtime
        .query(queries::estate_governance_context(ESTATE))
        .as_principal(observer)
        .controls(controls())
        .execute()
        .expect("the governance observer should read authoritative lifecycle state")
}

fn emergency(
    result: &GovernanceResult,
    access: EmergencyAccessId,
) -> &bank_domain::reads::EstateEmergencyContext {
    result.rows()[0]
        .capabilities()
        .iter()
        .find(|capability| capability.id() == GRANT)
        .expect("the original governed grant should remain observable")
        .emergencies()
        .iter()
        .find(|emergency| emergency.id() == access)
        .expect("the exact lifecycle record should be projected from current graph truth")
}

fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 1, 20_000).unwrap()
}

fn bank_idempotency(seed: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("active-elevation-{seed}")).unwrap()
}
