//! A producer's required rule holds over a publication Relational proves
//! leaves the rule's closure. A deletion's footprint is not proven, so the
//! same requirement refuses it, and the caller is told the receipt's basis.
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::program;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::world;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::Rule;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryGeneratedOutputInvariantAdmissionDenial;
use worth_query_execution::facade::application_contribution::WorthQueryProducerInvariantRequirement;
use worth_query_execution::domain_computation::primary_graph::denial;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind as Invariant;
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_relational::facade::mvcc::CustomInvariantNonApplicability;
use worth_relational::facade::runtime::InvariantDecisionKind;

pub const LEDGER: &[WorthQueryProducerInvariantRequirement] =
    &[WorthQueryProducerInvariantRequirement::new(
        "LedgerOnly",
        1,
        0,
        ApplicationInvariantExecutionPoint::CommitBoundary,
    )];

pub fn install() -> crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::world::World{
    let visits = Arc::new(Mutex::new(Vec::new()));
    let reads = Arc::new(Mutex::new(Vec::new()));
    let rule = |ordinal| Rule {
        ordinal,
        cancellation: None,
        visits: visits.clone(),
        reads: reads.clone(),
    };
    crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::world::install(rule(1), rule(2))
}

pub fn scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token())
}

pub fn commit<Operation, Input>(
    world: &crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::world::World,
    program: WorthQueryApplicationEffectProgram<
        crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::schema::RefusalSchema,
        Operation,
        Input,
        crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::schema::Account,
    >,
    key: u8,
) -> WorthQueryApplicationCommitOutcome
where
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    world.application.compare_and_commit_application(
        program,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(key, key),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

#[test]
fn a_write_outside_the_required_rule_closure_publishes() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = install();
    let program =
        crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::program::prepare(&world, &scope(), "open".into()).with_producer_required_invariants(LEDGER);
    let outcome = commit(&world, program, 211);
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "an account write leaves the ledger rule's closure: {outcome:?}"
    );
}

#[test]
fn a_deletion_outside_the_required_rule_closure_is_refused_with_its_basis() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = install();
    let program = crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::program::close(&world, &scope()).with_producer_required_invariants(LEDGER);
    let WorthQueryApplicationCommitOutcome::Denied(denial) = commit(&world, program, 212) else {
        panic!("an unproven deletion footprint must not satisfy the requirement");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution,
        "{denial:?}"
    );
    assert_eq!(
        denial
            .invariant_execution_failure()
            .map(|failure| failure.kind()),
        Some(Invariant::ProducerRequirementNotAdmitted(
            WorthQueryGeneratedOutputInvariantAdmissionDenial::RequiredInvariantDidNotPass {
                verdict: InvariantDecisionKind::NotApplicable,
                non_applicability: Some(CustomInvariantNonApplicability::NoApplicableTouch),
            }
        ))
    );

    // Without the requirement the same deletion publishes: the refusal is the
    // requirement's, not the deletion's.
    let program = crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::program::close(&world, &scope());
    let outcome = commit(&world, program, 213);
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the deletion itself is lawful: {outcome:?}"
    );
}
