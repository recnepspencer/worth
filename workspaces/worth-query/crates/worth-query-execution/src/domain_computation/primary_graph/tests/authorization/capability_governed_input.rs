use super::super::application_attempt::{authenticated_principal, idempotency};
use super::super::fixture::{
    installed_capability_authorization_world, live_scope, AuthorizationWorld as World,
    CapabilityTouchInput, CapabilityTouchMutationBinding, CapabilityTouchOperation,
    IdentityExecutionSchema,
};
use super::capability_progression::{
    admitted_capability_access, admitted_capability_program, admitted_capability_program_for_input,
    capability_input, time,
};
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerResult, WorthQueryCompletedMutationCandidate,
};
use crate::domain_computation::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitOutcome, WorthQueryAuthenticatedPrincipal,
    WorthQueryInvariantProjectionWork, WorthQueryMutationHandlerExecutionReport,
    WorthQueryMutationHandlerWork,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
};

type Principal = WorthQueryAuthenticatedPrincipal<
    IdentityExecutionSchema,
    super::super::fixture::Principal,
    u64,
>;
type TouchHandlerOutcome = WorthQueryMutationHandlerExecutionReport<
    WorthQueryCompletedMutationCandidate<IdentityExecutionSchema, CapabilityTouchMutationBinding>,
    CapabilityTouchInput,
>;

#[test]
fn every_capability_admission_reports_its_one_governed_input_derivation() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100); 24]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let (first, first_admission) =
        admitted_capability_program(&world, &principal, &request, "canonical-work");
    assert_one_canonical_derivation(first_admission.canonical_work);
    let (retry, retry_admission) =
        admitted_capability_program(&world, &principal, &request, "canonical-work");
    assert_eq!(
        retry_admission.canonical_work, first_admission.canonical_work,
        "the same input always reports the same deterministic work",
    );

    let WorthQueryApplicationCommitOutcome::Committed(committed) = world
        .application
        .compare_and_commit_application(first, idempotency(81, 81))
    else {
        panic!("the first governed-input attempt must commit");
    };
    assert_admission_only_work(committed.canonical_work(), first_admission.canonical_work);
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(81, 81))
    else {
        panic!("an equal input under the same key must recover the commit");
    };
    assert_admission_only_work(recovered.canonical_work(), retry_admission.canonical_work);
}

#[test]
fn governed_input_identity_drift_under_a_reused_key_is_denied() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100); 24]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let (first, _) = admitted_capability_program(&world, &principal, &request, "governed-input");
    let mut drifted_input = capability_input(100);
    drifted_input.amount -= 1;
    let (drifted, _) = admitted_capability_program_for_input(
        &world,
        &principal,
        &request,
        "governed-input",
        drifted_input,
    );

    let first_outcome = world
        .application
        .compare_and_commit_application(first, idempotency(83, 83));
    assert!(
        matches!(
            first_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "the first governed-input attempt must commit: {first_outcome:?}"
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(drifted, idempotency(83, 83))
    else {
        panic!("a changed serialized field under the same key must be denied");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}

#[test]
fn the_handler_runs_on_exactly_the_input_its_capability_admitted() {
    let (world, principal) = capability_world();
    let report = execute_touch_handler(&world, &principal, capability_input(100));
    let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
        panic!("the installed zero-read handler executed");
    };
    assert!(work.handler_contacted());
    assert_eq!(
        work.projection_work(),
        WorthQueryInvariantProjectionWork::default()
    );
    match report.into_outcome() {
        Ok(HandlerResult::Completed(_)) => {}
        Ok(_) => panic!("the handler accepts the input it is given"),
        Err(denial) => panic!("the admitted input must reach its installed handler: {denial:?}"),
    }
}

#[test]
fn a_request_whose_input_differs_from_the_admitted_input_is_refused() {
    let (world, principal) = capability_world();
    let mut other = capability_input(100);
    other.amount -= 1;
    let report = execute_touch_handler(&world, &principal, other);
    assert_eq!(
        *report.decision_work(),
        WorthQueryMutationHandlerWork::NotStarted
    );
    let outcome = report.into_outcome();
    assert!(
        matches!(
            outcome,
            Err(MutationHandlerExecutionDenial::InputNotAdmitted)
        ),
        "an input other than the admitted one must be refused before any handler runs"
    );
}

fn capability_world() -> (World, Principal) {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100); 24]);
    let principal = authenticated_principal(&world, &live_scope());
    (world, principal)
}

/// Admits `capability_input(100)` and runs the touch handler for a request
/// carrying `requested`.
fn execute_touch_handler(
    world: &World,
    principal: &Principal,
    requested: CapabilityTouchInput,
) -> TouchHandlerOutcome {
    let request = live_scope();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();
    let access = admitted_capability_access(world, principal, &request, 100).unwrap();
    let admission = world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .unwrap();
    assert!(admission.governed_input_identity().is_some());

    let key = "governed-input-handler".to_owned();
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        CapabilityTouchMutationBinding,
    >::encode(&key, &requested)
    .expect("the request encodes");
    world
        .application
        .execute_mutation_handler_report::<CapabilityTouchMutationBinding>(
            &identities,
            principal.principal_identity(),
            admission,
        )
}

/// The governed input, encoded once: 289 encoded bytes, hashed with 88 bytes
/// of framing (the 30-byte operation input domain and the 42-byte input type
/// `worth.query.test.capability-touch-input.v1`, each behind an 8-byte length)
/// into 377 SHA-256 input bytes, which pad to 7 compression blocks.
fn assert_one_canonical_derivation(work: WorthQueryCanonicalWorkEvidence) {
    assert_eq!(work.basis_preparations(), 0);
    assert_eq!(work.digest_derivations(), 1);
    assert_eq!(work.canonical_entries(), 1);
    assert_eq!(work.canonical_encoded_bytes(), 289);
    assert_eq!(work.canonical_material_allocation_bytes(), 0);
    assert_eq!(work.sha256_input_bytes(), 377);
    assert_eq!(work.sha256_compression_blocks(), 7);
    assert_eq!(work.digest_text_materializations(), 0);
}

fn assert_admission_only_work(
    phases: WorthQueryCanonicalWorkPhases,
    expected_admission: WorthQueryCanonicalWorkEvidence,
) {
    assert_eq!(phases.admission(), expected_admission);
    for phase in [
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.live_delivery(),
        phases.retry_resolution(),
        phases.recovery_inspection(),
        phases.publication(),
    ] {
        assert_eq!(phase, WorthQueryCanonicalWorkEvidence::zero());
    }
}
