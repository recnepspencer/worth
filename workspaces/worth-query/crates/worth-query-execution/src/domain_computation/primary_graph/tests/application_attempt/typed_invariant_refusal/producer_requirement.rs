//! A producer's required rule holds over a publication Relational proves
//! leaves the rule's closure. A deletion's footprint is not proven, so the
//! same requirement refuses it, and the caller is told the receipt's basis.
use super::{program, world, Rule};
use crate::domain_computation::primary_graph::*;
use crate::domain_computation::WorthQueryInvariantExecutionDenialKind as Invariant;
use std::sync::{Arc, Mutex};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_relational::facade::mvcc::CustomInvariantNonApplicability;
use worth_relational::facade::runtime::InvariantDecisionKind;

const LEDGER: &[WorthQueryProducerInvariantRequirement] =
    &[WorthQueryProducerInvariantRequirement::new(
        "LedgerOnly",
        1,
        0,
        ApplicationInvariantExecutionPoint::CommitBoundary,
    )];

fn install() -> world::World {
    let visits = Arc::new(Mutex::new(Vec::new()));
    let rule = |ordinal| Rule {
        ordinal,
        cancellation: None,
        visits: visits.clone(),
    };
    world::install(rule(1), rule(2))
}

fn scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token())
}

fn commit<Operation, Input>(
    world: &world::World,
    program: WorthQueryApplicationEffectProgram<
        super::schema::RefusalSchema,
        Operation,
        Input,
        super::schema::Account,
    >,
    key: u8,
) -> WorthQueryApplicationCommitOutcome
where
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    world.application.compare_and_commit_application(
        program,
        super::super::idempotency(key, key),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

#[test]
fn a_write_outside_the_required_rule_closure_publishes() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let world = install();
    let program =
        program::prepare(&world, &scope(), "open".into()).with_producer_required_invariants(LEDGER);
    let outcome = commit(&world, program, 211);
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "an account write leaves the ledger rule's closure: {outcome:?}"
    );
}

#[test]
fn a_deletion_outside_the_required_rule_closure_is_refused_with_its_basis() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let world = install();
    let program = program::close(&world, &scope()).with_producer_required_invariants(LEDGER);
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
    let program = program::close(&world, &scope());
    let outcome = commit(&world, program, 213);
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the deletion itself is lawful: {outcome:?}"
    );
}
