use std::time::Duration;

use bank_domain::{
    estate::{
        CapabilityGrantId, CapabilityGrantStatus, EmergencyAccessId, EstateAction,
        EstateWorkflowStage, RestrictedBankField,
    },
    queries::EstateGovernanceQuery,
    reads::{EstateCapabilityContext, EstateGovernanceContext},
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationResult;

use super::{
    fixture::{
        emergency_request_world_with_alternate_bound, request_scope, CapabilityFixture, GrantSpec,
        ALTERNATE_EMERGENCY_BOUND_GRANT, ESTATE, GRANT,
    },
    lifecycle_journey::{
        approve_elevation, request_elevation, ElevationApprovalSpec, ElevationRequestSpec,
    },
};
use crate::{
    queries, BankApplicationQueryDenial, BankApprovedEstateElevation, BankAuthenticatedPrincipal,
    BankMutationCommitOutcome, BankReadControls,
};

type GovernanceResult =
    WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>;

#[test]
fn approved_emergency_historical_and_preview_preserve_one_shot_meaning() {
    let fixture = lane_world("estate-emergency-lane-parity");
    let (requester, approved) = approve(&fixture, 401, 402, 131);
    let request =
        queries::estate_emergency_account_details(ESTATE, EmergencyAccessId::new(401).unwrap());
    let one_shot = fixture
        .runtime
        .query(request)
        .as_principal(&requester)
        .controls(controls())
        .execute_with_approved_elevation(&approved)
        .expect("one-shot establishes the canonical emergency result meaning");
    let historical = fixture
        .runtime
        .query(request)
        .as_principal(&requester)
        .controls(controls())
        .admit_historical_with_approved_elevation(&approved, |admitted| admitted.execute())
        .expect("historical execution should preserve approved one-shot meaning");
    let session = fixture.runtime.open_preview(&request_scope()).unwrap();
    let preview = fixture
        .runtime
        .query(request)
        .as_principal(&requester)
        .controls(controls())
        .admit_preview_with_approved_elevation(&approved, &session, |admitted| admitted.execute())
        .expect("preview execution should preserve approved one-shot meaning");

    assert_eq!(historical.rows(), one_shot.rows());
    assert_eq!(preview.rows(), one_shot.rows());
    assert!(one_shot.receipt().inspect().terminal_resources_released());
    assert!(historical.receipt().inspect().terminal_resources_released());
    assert!(preview.receipt().inspect().terminal_resources_released());
    assert!(session.discard().unwrap().discarded());
    assert_resources_released(&fixture);
}

#[test]
fn abandoning_an_admitted_lane_releases_its_scoped_authority() {
    let fixture = lane_world("estate-emergency-abandoned-lanes");
    let (requester, approved) = approve(&fixture, 406, 407, 136);
    let request =
        queries::estate_emergency_account_details(ESTATE, EmergencyAccessId::new(406).unwrap());
    fixture
        .runtime
        .query(request)
        .as_principal(&requester)
        .controls(controls())
        .admit_historical_with_approved_elevation(&approved, |_admitted| Ok(()))
        .expect("dropping historical authority without execution is lawful");
    assert_resources_released(&fixture);

    let session = fixture.runtime.open_preview(&request_scope()).unwrap();
    fixture
        .runtime
        .query(request)
        .as_principal(&requester)
        .controls(controls())
        .admit_preview_with_approved_elevation(&approved, &session, |_admitted| Ok(()))
        .expect("dropping preview authority without execution is lawful");
    assert!(session.discard().unwrap().discarded());
    assert_resources_released(&fixture);
}

#[test]
fn historical_denies_if_exact_support_is_revoked_after_admission() {
    let fixture = lane_world("estate-emergency-historical-cutoff");
    let (requester, approved) = approve(&fixture, 411, 412, 141);
    let denial = match fixture
        .runtime
        .query(queries::estate_emergency_account_details(
            ESTATE,
            EmergencyAccessId::new(411).unwrap(),
        ))
        .as_principal(&requester)
        .controls(controls())
        .admit_historical_with_approved_elevation(&approved, |admitted| {
            revoke_exact_support(&fixture, &requester, 145);
            admitted.execute()
        }) {
        Ok(_) => panic!("historical delivery must refresh exact support after admission"),
        Err(denial) => denial,
    };

    let BankApplicationQueryDenial::Execution(denial) = denial else {
        panic!("historical cutoff must occur during selected execution: {denial:?}");
    };
    assert_stale_authorization(denial.kind());
    assert_exact_revoked_alternate_active(&fixture, &requester);
    assert_resources_released(&fixture);
}

#[test]
fn preview_denies_if_exact_support_is_revoked_after_admission() {
    let fixture = lane_world("estate-emergency-preview-cutoff");
    let (requester, approved) = approve(&fixture, 421, 422, 151);
    let session = fixture.runtime.open_preview(&request_scope()).unwrap();
    let denial = match fixture
        .runtime
        .query(queries::estate_emergency_account_details(
            ESTATE,
            EmergencyAccessId::new(421).unwrap(),
        ))
        .as_principal(&requester)
        .controls(controls())
        .admit_preview_with_approved_elevation(&approved, &session, |admitted| {
            revoke_exact_support(&fixture, &requester, 155);
            admitted.execute()
        }) {
        Ok(_) => panic!("preview delivery must refresh exact support after admission"),
        Err(denial) => denial,
    };

    let BankApplicationQueryDenial::Execution(denial) = denial else {
        panic!("preview cutoff must occur during selected execution: {denial:?}");
    };
    assert_stale_authorization(denial.kind());
    assert_exact_revoked_alternate_active(&fixture, &requester);
    assert!(session.discard().unwrap().discarded());
    assert_resources_released(&fixture);
}

fn lane_world(scenario: &str) -> CapabilityFixture {
    emergency_request_world_with_alternate_bound(
        scenario,
        GrantSpec::emergency_view(),
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    )
}

fn approve(
    fixture: &CapabilityFixture,
    access: u64,
    review: u64,
    idempotency_seed: u8,
) -> (BankAuthenticatedPrincipal, BankApprovedEstateElevation) {
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let requested = request_elevation(
        fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access,
            review,
            idempotency: idempotency_seed,
            field: RestrictedBankField::AccountDetails,
            duration: Duration::from_secs(300),
        },
    );
    let approved = approve_elevation(
        fixture,
        &approver,
        requested,
        ElevationApprovalSpec {
            access,
            idempotency: idempotency_seed + 2,
        },
    );
    (requester, approved)
}

fn revoke_exact_support(
    fixture: &CapabilityFixture,
    principal: &BankAuthenticatedPrincipal,
    idempotency_seed: u8,
) {
    let outcome = fixture
        .runtime
        .revoke_estate_capability(
            principal,
            EstateAction::RevokeCapability {
                estate: ESTATE,
                grant: GRANT,
            },
            WorthQueryApplicationIdempotencyBinding::new(
                [idempotency_seed; 32],
                [idempotency_seed + 1; 32],
            ),
            &request_scope(),
        )
        .expect("the exact support revocation should execute after query admission");
    assert!(matches!(outcome, BankMutationCommitOutcome::Committed(_)));
}

fn assert_stale_authorization(kind: crate::BankApplicationOneShotDenialKind) {
    assert_eq!(
        kind,
        crate::BankApplicationOneShotDenialKind::Authorization(
            crate::BankAuthorizationDenialKind::StaleAuthorization,
        )
    );
}

fn assert_exact_revoked_alternate_active(
    fixture: &CapabilityFixture,
    observer: &BankAuthenticatedPrincipal,
) {
    let observed = governance_readback(fixture, observer);
    assert_eq!(
        capability(&observed, GRANT).status(),
        CapabilityGrantStatus::Revoked
    );
    assert_eq!(
        capability(&observed, ALTERNATE_EMERGENCY_BOUND_GRANT).status(),
        CapabilityGrantStatus::Active,
        "an equivalent current grant must not replace the admitted exact support"
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
        .expect("governance authority should independently observe current grant status")
}

fn capability(result: &GovernanceResult, grant: CapabilityGrantId) -> &EstateCapabilityContext {
    result.rows()[0]
        .capabilities()
        .iter()
        .find(|capability| capability.id() == grant)
        .expect("the exact capability grant should be visible")
}

fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 1, 20_000).unwrap()
}

fn assert_resources_released(fixture: &CapabilityFixture) {
    let application = fixture.runtime.application_runtime();
    assert_eq!(
        application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    let buffers = application.result_buffer_observer().observe();
    assert_eq!(buffers.active_buffers(), 0);
    assert_eq!(buffers.retained_bytes(), 0);
}
