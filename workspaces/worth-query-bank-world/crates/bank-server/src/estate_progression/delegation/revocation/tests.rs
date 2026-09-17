use bank_domain::proposals::BankIdempotencyKey;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding,
};

use super::*;
use crate::estate_capability_admission::fixture::{
    emergency_request_world_with_alternate_bound, request_scope, GrantSpec, ESTATE, GRANT,
};

#[test]
fn generic_program_cannot_bypass_the_installed_revocation_action() {
    let fixture = revocation_world("capability-revocation-generic-bypass");
    let specialist = fixture.authenticate();
    let action = revocation_action();
    let admission = fixture
        .runtime
        .admit_capability_revocation(&specialist, action, &request_scope())
        .expect("the exact command and target should admit specialized revocation");
    let program = generic_empty_program(&fixture.runtime, admission, GRANT);
    let outcome = fixture
        .runtime
        .application_runtime()
        .compare_and_commit_application(program, query_idempotency(151));
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("generic revocation must deny before provider execution: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProposalBinding
    );
}

#[test]
fn target_status_drift_after_materialization_stales_provider_commit() {
    let fixture = revocation_world("capability-revocation-provider-currentness");
    let specialist = fixture.authenticate();
    let action = revocation_action();
    let admission = fixture
        .runtime
        .admit_capability_revocation(&specialist, action, &request_scope())
        .expect("the exact command should admit while the target is active");
    let program = fixture
        .runtime
        .materialize_capability_revocation(admission, GRANT)
        .expect("the exact Active -> Revoked program should materialize");

    let drift = fixture
        .runtime
        .revoke_estate_capability_with_key(&specialist, action, &idempotency(153), &request_scope())
        .expect("a separate public command should revoke the prepared target");
    assert!(matches!(drift, BankMutationCommitOutcome::Committed(_)));

    let outcome = fixture
        .runtime
        .application_program()
        .admit_program_operation::<RevokeEstateCapabilityOperation>()
        .unwrap()
        .compare_and_commit_capability_revocation(program, query_idempotency(155));
    assert_product_basis_stale(outcome);
}

fn assert_product_basis_stale(outcome: WorthQueryApplicationCommitOutcome) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("an old materialized program must retain its exact product basis: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
}

#[test]
fn exact_revocation_retry_replays_after_the_grant_is_revoked() {
    let fixture = revocation_world("capability-revocation-exact-replay");
    let specialist = fixture.authenticate();
    let action = revocation_action();
    let key = idempotency(159);
    let first = fixture
        .runtime
        .revoke_estate_capability_with_key(&specialist, action, &key, &request_scope())
        .expect("the active grant should revoke");
    assert!(matches!(first, BankMutationCommitOutcome::Committed(_)));
    let replay = fixture
        .runtime
        .revoke_estate_capability_with_key(&specialist, action, &key, &request_scope())
        .expect("the same request should replay after revocation");
    assert!(matches!(
        replay,
        BankMutationCommitOutcome::AlreadyCommitted(_)
    ));
}

/// Q8.26-C1: the retention demand attaches from the installed contract, with no
/// per-operation opt-in that could omit it.
///
/// `RevokeCapability` declares `RecordedInverse` with `ExactPriorTruth` over
/// `CapabilityGrantStatusField`, but it commits through the specialized
/// capability-revocation program whose constructor hard-coded
/// `preimage_demand: None`. Only `FreezeAccount` ever called the public
/// `with_preimage_demand` builder, so this operation declared a correction
/// mechanism and retained nothing to correct with. No existing test noticed,
/// because "did it commit" is blind to an empty pre-image — the two tests above
/// both commit revocations and neither can see it.
#[test]
fn revocation_retains_its_declared_preimage_without_a_per_operation_opt_in() {
    let fixture = revocation_world("capability-revocation-retains-preimage");
    let specialist = fixture.authenticate();
    let outcome = fixture
        .runtime
        .revoke_estate_capability_with_key(
            &specialist,
            revocation_action(),
            &idempotency(161),
            &request_scope(),
        )
        .expect("the exact command should revoke the active target");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("revocation must commit: {outcome:?}");
    };
    assert!(
        receipt.retained_preimage(),
        "RevokeCapability declares RecordedInverse/ExactPriorTruth, so its commit \
         must carry the pre-image its installed contract demands"
    );
    assert!(receipt.performed_preimage_retention_work());
}

fn generic_empty_program(
    runtime: &BankIdentityRuntime,
    admission: AdmittedCapabilityRevocation,
    grant: CapabilityGrantId,
) -> WorthQueryApplicationEffectProgram<
    BankSchema,
    RevokeEstateCapabilityOperation,
    EstateAction,
    EstateCase,
> {
    let projected = runtime
        .invariant_projection()
        .project_admitted_operation(&admission, |reader, estate| {
            project_active_estate_grant(reader, estate, grant)
        })
        .unwrap();
    let (result, projection, _) = projected.into_parts();
    result.unwrap();
    let reads = runtime
        .application_runtime()
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    reads
        .resolve_entity(CapabilityGrantIdentityField::reference(), grant)
        .unwrap();
    reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program()
        .finish()
        .unwrap()
}

fn revocation_world(
    scenario: &str,
) -> crate::estate_capability_admission::fixture::CapabilityFixture {
    emergency_request_world_with_alternate_bound(
        scenario,
        GrantSpec::emergency_view(),
        GrantSpec::emergency_view(),
        bank_domain::estate::EstateWorkflowStage::Administration,
    )
}

fn revocation_action() -> EstateAction {
    EstateAction::RevokeCapability {
        estate: ESTATE,
        grant: GRANT,
    }
}

fn idempotency(seed: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("capability-revocation-owner-{seed}")).unwrap()
}

fn query_idempotency(seed: u8) -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new([seed; 32], [seed + 1; 32])
}
