use std::sync::atomic::Ordering;

use worth_query_consumer_values::{PlanarAdjustment, PlanarAdjustmentResult, PlanarOperation};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequest,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
        WorthQueryApplicationCommitOutcome, WorthQueryCustomInvariantDenial,
    },
};
use worth_query_topology_entry::{PlanarMutation, PlanarRead};

use super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

mod output_correspondence;
mod publication;
mod resource_profile;

type Request<'a> = WorthQueryApplicationRequest<'a, 'a, 'a, ConsumerSchema>;
type MutationOutcome = WorthQueryApplicationMutationOutcome<
    worth_query_consumer_values::PlanarMutationDenial,
    PlanarAdjustmentResult,
>;

pub(crate) fn run(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    super::contribution_denials::run();
    resource_profile::candidate_bytes_beyond_host_limit_are_denied(foreign);
    let world = installation::install(foreign);
    super::discovery::verify(&world.application);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the installed adapter authenticates the issued local credential");
    let request = world.application.request(&principal, &scope);
    assert_eq!(read_y(&request, "anchor-a"), 1);
    assert_eq!(read_y(&request, "sibling-a"), 21);

    typed_domain_denial_has_no_publication(&request, &world);
    insufficient_work_is_denied_before_owner(&request, &world);
    publication::create_and_reject_cycles(&request);
    actual_candidate_checks_untouched_neighbors(&request, &world);
    output_correspondence::run(&request);
    println!("Pre-M0 public candidate journey passed: variable cyclic allocation, atomic publication/read, exact invariant denial, early work exhaustion, idempotency and untouched-neighbor closure");
}

fn typed_domain_denial_has_no_publication(
    request: &Request<'_>,
    world: &installation::ConsumerWorld,
) {
    let before = source_version(request);
    let calls = world.invariant_calls.load(Ordering::SeqCst);
    let outcome = mutate(
        request,
        PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::CreateCycle(Vec::new()),
            validator_work: 4096,
        },
        2,
    );
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::DomainDenied(
            worth_query_consumer_values::PlanarMutationDenial::CycleNeedsThreeVertices
        )
    ));
    assert_eq!(source_version(request), before);
    assert_eq!(world.invariant_calls.load(Ordering::SeqCst), calls);
}

fn insufficient_work_is_denied_before_owner(
    request: &Request<'_>,
    world: &installation::ConsumerWorld,
) {
    let calls = world.invariant_calls.load(Ordering::SeqCst);
    let before = source_version(request);
    let outcome = mutate(request, adjust("anchor-a", 2, 1), 1);
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationCommitOutcome::Denied(
        denial,
    )) = outcome
    else {
        panic!("insufficient validator work must deny at invariant admission: {outcome:?}")
    };
    assert!(matches!(denial.kind(),
        WorthQueryApplicationCommitDenialKind::CandidateValidatorWorkExceeded {
            maximum_work: 1, required_work,
        } if required_work > 1
    ));
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        world.invariant_calls.load(Ordering::SeqCst),
        calls,
        "a rejected work reservation must not invoke the custom rule"
    );
    assert_eq!(source_version(request), before);
    assert_eq!(read_y(request, "anchor-a"), 1);
}

fn actual_candidate_checks_untouched_neighbors(
    request: &Request<'_>,
    world: &installation::ConsumerWorld,
) {
    let calls = world.invariant_calls.load(Ordering::SeqCst);
    let valid = adjust("anchor-a", 2, 4096);
    let outcome = mutate(request, valid.clone(), 10);
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("the positive-turn adjustment must commit: {outcome:?}")
    };
    assert_eq!(result.changed_vertices, 1);
    assert!(world.invariant_calls.load(Ordering::SeqCst) > calls);
    assert_eq!(read_y(request, "anchor-a"), 2);
    assert_eq!(read_y(request, "anchor-b"), 1);
    assert_eq!(read_y(request, "anchor-c"), 10);

    let retry = mutate(request, valid, 10);
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the repeated command must recover its single committed publication")
    };
    assert!(recovered.is_same_authoritative_commit(&receipt));

    let before = source_version(request);
    let malformed = mutate(request, adjust("anchor-a", 12, 4096), 11);
    require_planar_violation(malformed);
    assert_eq!(
        source_version(request),
        before,
        "a rejected candidate does not advance the published graph version"
    );
    assert_eq!(
        read_y(request, "anchor-a"),
        2,
        "the rejected coordinate must remain invisible"
    );
    assert_eq!(read_y(request, "anchor-b"), 1);
    assert_eq!(read_y(request, "anchor-c"), 10);
    assert_eq!(read_y(request, "sibling-a"), 21);
}

fn require_planar_violation<Denial: std::fmt::Debug, Result: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<Denial, Result>,
) {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationCommitOutcome::Denied(
        denial,
    )) = outcome
    else {
        panic!("the malformed candidate must reach the installed invariant: {outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let Some(WorthQueryCustomInvariantDenial::Violation { identity }) =
        denial.custom_invariant_denial()
    else {
        panic!("the denial must carry the domain violation, not an execution failure")
    };
    assert_eq!(identity.rule_id.as_str(), "PositivePlanarTurn");
    assert_eq!(
        (
            identity.semantic_version.major,
            identity.semantic_version.minor
        ),
        (1, 0)
    );
}

fn mutate(request: &Request<'_>, input: PlanarMutation, key: u64) -> MutationOutcome {
    request
        .mutate(input)
        .idempotency(&key)
        .execute()
        .expect("the typed request reaches the actual mutation owner")
}

fn adjust(key: &str, y: u64, validator_work: usize) -> PlanarMutation {
    PlanarMutation {
        scope_key: "anchor-a".to_owned(),
        operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
            body_key: key.to_owned(),
            replacement_y: length(y),
        }]),
        validator_work,
    }
}

fn read_y(request: &Request<'_>, key: &str) -> u64 {
    let result = request
        .query(PlanarRead {
            body_key: key.to_owned(),
        })
        .execute()
        .expect("the published vertex is readable through its typed query");
    assert_eq!(result.rows().len(), 1);
    worth_query_consumer_values::PositiveLength::get(&result.rows()[0].y)
}

fn source_version(request: &Request<'_>) -> u64 {
    let result = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the selected source basis remains readable");
    result.receipt().inspect().basis().version()
}
