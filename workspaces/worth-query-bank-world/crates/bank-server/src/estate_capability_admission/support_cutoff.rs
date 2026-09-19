use std::time::Duration;

use bank_domain::{
    estate::{
        CapabilityGrantId, CapabilityGrantStatus, EmergencyAccessId, EmergencyAccessStatus,
        EstateAction, EstateWorkflowStage, MandatoryReviewId, MandatoryReviewStatus,
        RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
    queries::EstateGovernanceQuery,
    reads::{EstateCapabilityContext, EstateEmergencyContext, EstateGovernanceContext},
};
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationResult;

use super::{
    fixture::{
        emergency_request_world_with_alternate_bound, request_scope, CapabilityFixture, GrantSpec,
        ALTERNATE_EMERGENCY_BOUND_GRANT, ESTATE, GRANT, REVIEWER,
    },
    lifecycle_journey::{
        approve_elevation, request_elevation, ElevationApprovalSpec, ElevationRequestSpec,
    },
};
use crate::{
    queries, BankApplicationQueryDenial, BankAuthenticatedPrincipal,
    BankEstateElevationCloseOutcome, BankEstateMandatoryReviewOutcome, BankEstateProgressionDenial,
    BankMutationCommitOutcome, BankReadControls,
};

type GovernanceResult =
    WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>;

#[test]
fn revoked_support_cuts_active_use_but_not_close_or_mandatory_review() {
    let fixture = cutoff_world("estate-emergency-support-cutoff");
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let reviewer = fixture.authenticate_reviewer();
    let access = EmergencyAccessId::new(381).unwrap();
    let review = MandatoryReviewId::new(382).unwrap();
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 381,
            review: 382,
            idempotency: 111,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let approved = approve_elevation(
        &fixture,
        &approver,
        requested,
        ElevationApprovalSpec {
            access: 381,
            idempotency: 113,
        },
    );

    let published = fixture
        .runtime
        .query(queries::estate_emergency_account_details(ESTATE, access))
        .as_principal(&requester)
        .controls(controls())
        .execute_with_approved_elevation(&approved)
        .expect("the active exact support should permit emergency account details");
    assert_eq!(published.rows().len(), 1);

    revoke_exact_support(&fixture, &requester, 115);
    let revoked = governance_readback(&fixture, &requester);
    assert_eq!(
        capability(&revoked, GRANT).status(),
        CapabilityGrantStatus::Revoked
    );
    assert_eq!(
        capability(&revoked, ALTERNATE_EMERGENCY_BOUND_GRANT).status(),
        CapabilityGrantStatus::Active,
        "an equivalent grant must remain current so identity substitution would be observable"
    );

    let denial = match fixture
        .runtime
        .query(queries::estate_emergency_account_details(ESTATE, access))
        .as_principal(&requester)
        .controls(controls())
        .execute_with_approved_elevation(&approved)
    {
        Ok(_) => panic!("revoking the exact carried support must cut an already-approved use"),
        Err(denial) => denial,
    };
    let BankApplicationQueryDenial::CapabilityAdmission(denial) = denial else {
        panic!("support cutoff must fail at capability admission: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::StaleAuthorization
    );

    let closed = fixture
        .runtime
        .revoke_estate_emergency_access_with_key(
            &approver,
            approved,
            EstateAction::RevokeEmergencyAccess {
                estate: ESTATE,
                access,
            },
            &bank_idempotency(117),
            &request_scope(),
        )
        .expect("independent close authority must remain available after support revocation");
    let BankEstateElevationCloseOutcome::Closed(mandatory) = closed else {
        panic!("the exact approved receipt must close once: {closed:?}");
    };
    let reviewed = fixture
        .runtime
        .complete_estate_mandatory_review_with_key(
            &reviewer,
            mandatory,
            EstateAction::CompleteMandatoryReview {
                estate: ESTATE,
                access,
                review,
            },
            &bank_idempotency(119),
            &request_scope(),
        )
        .expect("independent review authority must survive support revocation");
    let BankEstateMandatoryReviewOutcome::Reviewed(_) = reviewed else {
        panic!("the mandatory review must complete once: {reviewed:?}");
    };

    let terminal = governance_readback(&fixture, &requester);
    let terminal_emergency = emergency(capability(&terminal, GRANT), access);
    assert_eq!(terminal_emergency.grant(), GRANT);
    assert_eq!(terminal_emergency.status(), EmergencyAccessStatus::Revoked);
    assert_eq!(
        terminal_emergency.mandatory_review().status,
        MandatoryReviewStatus::Completed
    );
    assert_eq!(terminal_emergency.reviewer(), Some(REVIEWER));
    assert_eq!(
        terminal_emergency.mandatory_review().reviewer,
        Some(REVIEWER)
    );
}

#[test]
fn revoked_request_support_cannot_be_replaced_during_approval() {
    let fixture = cutoff_world("estate-emergency-approval-support-cutoff");
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let access = EmergencyAccessId::new(391).unwrap();
    let requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: 391,
            review: 392,
            idempotency: 121,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    revoke_exact_support(&fixture, &requester, 123);

    let denial = fixture
        .runtime
        .approve_estate_emergency_access_with_key(
            &approver,
            requested,
            EstateAction::ApproveEmergencyAccess {
                estate: ESTATE,
                access,
            },
            &bank_idempotency(125),
            &request_scope(),
        )
        .expect_err("an equivalent grant must not replace the request's exact revoked support")
        .into_denial();
    let BankEstateProgressionDenial::ApprovalAuthorization(denial) = denial else {
        panic!("stale request support must fail during approval authorization: {denial:?}");
    };
    assert_eq!(
        denial.kind(),
        crate::BankAuthorizationDenialKind::StaleAuthorization
    );

    let observed = governance_readback(&fixture, &requester);
    assert_eq!(
        capability(&observed, GRANT).status(),
        CapabilityGrantStatus::Revoked
    );
    assert_eq!(
        capability(&observed, ALTERNATE_EMERGENCY_BOUND_GRANT).status(),
        CapabilityGrantStatus::Active
    );
    let pending = emergency(capability(&observed, GRANT), access);
    assert_eq!(pending.grant(), GRANT);
    assert_eq!(pending.status(), EmergencyAccessStatus::Requested);
    assert_eq!(pending.approver(), None);
}

fn cutoff_world(scenario: &str) -> CapabilityFixture {
    emergency_request_world_with_alternate_bound(
        scenario,
        GrantSpec::emergency_view(),
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    )
}

fn revoke_exact_support(
    fixture: &CapabilityFixture,
    principal: &BankAuthenticatedPrincipal,
    idempotency_seed: u8,
) {
    let outcome = fixture
        .runtime
        .revoke_estate_capability_with_key(
            principal,
            EstateAction::RevokeCapability {
                estate: ESTATE,
                grant: GRANT,
            },
            &bank_idempotency(idempotency_seed),
            &request_scope(),
        )
        .expect("the independent capability-revocation command should execute");
    assert!(
        matches!(outcome, BankMutationCommitOutcome::Committed(_)),
        "the exact support revocation must commit once: {outcome:?}"
    );
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
        .expect("governance authority should independently observe current graph truth")
}

fn capability(result: &GovernanceResult, grant: CapabilityGrantId) -> &EstateCapabilityContext {
    result.rows()[0]
        .capabilities()
        .iter()
        .find(|capability| capability.id() == grant)
        .expect("the exact capability grant should be visible to governance readback")
}

fn emergency(
    capability: &EstateCapabilityContext,
    access: EmergencyAccessId,
) -> &EstateEmergencyContext {
    capability
        .emergencies()
        .iter()
        .find(|emergency| emergency.id() == access)
        .expect("the exact emergency lifecycle should be visible under its governed grant")
}

fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 1, 20_000).unwrap()
}

fn bank_idempotency(seed: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("support-cutoff-{seed}")).unwrap()
}
