use std::time::Duration;

use bank_domain::{
    estate::{
        CapabilityGrantId, CapabilityGrantStatus, EmergencyAccessId, EstateAction,
        EstateWorkflowStage, RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
    queries::{
        estate_emergency_access_activity, EstateEmergencyAccessActivityItem,
        EstateEmergencyAccessActivityRequest, EstateGovernanceQuery,
    },
    reads::{EstateCapabilityContext, EstateGovernanceContext},
};
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationResult;

use super::super::{
    fixture::{
        emergency_request_world_with_alternate_bound, request_scope, CapabilityFixture, GrantSpec,
        ALTERNATE_EMERGENCY_BOUND_GRANT, ESTATE, GRANT,
    },
    lifecycle_journey::{
        approve_elevation, request_elevation, ElevationApprovalSpec, ElevationRequestSpec,
    },
};
use crate::{
    queries, BankApprovedEstateElevation, BankAuthenticatedPrincipal, BankMutationCommitOutcome,
    BankReadControls, BankRequestedEstateElevation,
};

pub(super) const FIRST_ACCESS: u64 = 601;
pub(super) const SECOND_ACCESS: u64 = 602;

pub(super) struct ActivityWorld {
    pub(super) fixture: CapabilityFixture,
    pub(super) requester: BankAuthenticatedPrincipal,
    pub(super) approver: BankAuthenticatedPrincipal,
    pub(super) first_requested: Option<BankRequestedEstateElevation>,
    pub(super) approved: BankApprovedEstateElevation,
}

pub(super) fn activity_world(scenario: &str) -> ActivityWorld {
    let field = RestrictedBankField::EmergencyAccessActivity;
    let fixture = emergency_request_world_with_alternate_bound(
        scenario,
        GrantSpec::emergency_view_for(field),
        GrantSpec::emergency_view_for(field),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let first_requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: FIRST_ACCESS,
            review: 701,
            idempotency: 141,
            field,
            duration: Duration::from_secs(300),
        },
    );
    let second_requested = request_elevation(
        &fixture,
        &requester,
        ElevationRequestSpec {
            grant: GRANT,
            access: SECOND_ACCESS,
            review: 702,
            idempotency: 143,
            field,
            duration: Duration::from_secs(300),
        },
    );
    let approved = approve_elevation(
        &fixture,
        &approver,
        second_requested,
        ElevationApprovalSpec {
            access: SECOND_ACCESS,
            idempotency: 145,
        },
    );
    ActivityWorld {
        fixture,
        requester,
        approver,
        first_requested: Some(first_requested),
        approved,
    }
}

pub(super) fn activity_request() -> EstateEmergencyAccessActivityRequest {
    estate_emergency_access_activity(ESTATE, EmergencyAccessId::new(SECOND_ACCESS).unwrap())
}

pub(super) fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results, 20_000).unwrap()
}

pub(super) fn take_first_requested(world: &mut ActivityWorld) -> BankRequestedEstateElevation {
    world
        .first_requested
        .take()
        .expect("the first requested lifecycle should be approved once")
}

pub(super) fn approve_first(world: &ActivityWorld, requested: BankRequestedEstateElevation) {
    let _ = approve_elevation(
        &world.fixture,
        &world.approver,
        requested,
        ElevationApprovalSpec {
            access: FIRST_ACCESS,
            idempotency: 149,
        },
    );
}

pub(super) fn revoke_exact_support(world: &ActivityWorld, seed: u8) {
    let outcome = world
        .fixture
        .runtime
        .revoke_estate_capability_with_key(
            &world.requester,
            EstateAction::RevokeCapability {
                estate: ESTATE,
                grant: GRANT,
            },
            &BankIdempotencyKey::new(format!("activity-support-revocation-{seed}")).unwrap(),
            &request_scope(),
        )
        .expect("the exact activity support revocation should commit");
    assert!(matches!(outcome, BankMutationCommitOutcome::Committed(_)));
}

pub(super) fn assert_exact_revoked_alternate_active(world: &ActivityWorld) {
    let result: WorthQueryPublishedApplicationResult<
        EstateGovernanceQuery,
        EstateGovernanceContext,
    > = world
        .fixture
        .runtime
        .query(queries::estate_governance_context(ESTATE))
        .as_principal(&world.requester)
        .controls(controls(1))
        .execute()
        .expect("governance readback should expose exact support identity");
    assert_eq!(
        capability(&result, GRANT).status(),
        CapabilityGrantStatus::Revoked
    );
    assert_eq!(
        capability(&result, ALTERNATE_EMERGENCY_BOUND_GRANT).status(),
        CapabilityGrantStatus::Active
    );
}

pub(super) fn assert_resources_released(world: &ActivityWorld) {
    let application = world.fixture.runtime.application_runtime();
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
    assert!(buffers.peak_observed_bytes() > 0);
}

pub(super) fn items(
    rows: &[bank_domain::queries::EstateEmergencyAccessActivity],
) -> Vec<EstateEmergencyAccessActivityItem> {
    rows.iter()
        .flat_map(|row| row.accesses().iter().copied())
        .collect()
}

fn capability(
    result: &WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>,
    grant: CapabilityGrantId,
) -> &EstateCapabilityContext {
    result.rows()[0]
        .capabilities()
        .iter()
        .find(|capability| capability.id() == grant)
        .expect("the exact grant should be visible")
}
