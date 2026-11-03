//! The performed proofs a retained record holds and a republication keeps.

use std::sync::Arc;
use std::sync::OnceLock;

use crate::domain_computation::primary_graph::output_lineage::tests::republication::Witness;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::handler_fact_boundary::CompletedHandlerFactBoundary;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::decision_reuse::PreparedDecisionReuseContext;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
use worth_query_execution::facade::application_contribution::WorthQueryDecisionContextDependencies;
use worth_query_execution::facade::application_contribution::WorthQueryProducerInputReuseContract;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_reuse_key::PreparedInputReuseKey;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::decision_context_use::DecisionContextUse;

pub const INPUT: [u8; 32] = [0x61; 32];
pub const DECISION_KEY: [u8; 32] = [0x71; 32];

pub fn decision(key: [u8; 32]) -> PreparedDecisionReuseContext {
    PreparedDecisionReuseContext::new(
        WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::KEY,
        ),
        Some(key),
        None,
        None,
    )
    .unwrap()
}

pub fn input_key(
    selection: &WorthQueryObservedSourceSelection,
    input: [u8; 32],
) -> PreparedInputReuseKey {
    PreparedInputReuseKey::new(
        selection.clone(),
        input,
        InstalledProducerEdition::for_test([0x51; 32]),
    )
}

/// A performed origin: one handler fact, then its source selection.
pub fn performed(
    row: RecordedOutput,
    selection: &WorthQueryObservedSourceSelection,
    consumed: &Arc<[ConsumedOutputEvidence]>,
    witness: &Witness,
) -> RecordedOutput {
    let boundary = CompletedHandlerFactBoundary::completed_for_test(1);
    let completed = boundary
        .seal_decision_reuse(
            decision(DECISION_KEY),
            DecisionContextUse::default().key_for_test(),
        )
        .unwrap();
    RecordedOutput {
        producing_implementation: None,
        required_publication: None,
        native_prior_checkpoint: None,
        computation_source: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false),
        consumed_outputs: Arc::clone(consumed),
        completed_handler_facts: Some(boundary),
        completed_decision_reuse: Some(completed),
        prepared_input_reuse_key: Some(input_key(selection, INPUT)),
        native_output_witness: OnceLock::from(Arc::clone(witness)),
        ..row
    }
}

pub fn assert_continues_the_performed_read(row: &RecordedOutput) {
    assert!(row.performed_origin.is_none());
    assert_eq!(row.verification_requirement(), None);
    assert_eq!(
        row.completed_handler_facts
            .as_ref()
            .unwrap()
            .handler_fact_count(),
        1
    );
    let completed = row.completed_decision_reuse.as_ref().unwrap();
    let same = |key| decision(key).matches_completed(completed);
    assert!(same(DECISION_KEY));
    assert!(!same([0x72; 32]));
}

pub fn continues_input(
    row: &RecordedOutput,
    selection: &WorthQueryObservedSourceSelection,
    input: [u8; 32],
) -> bool {
    row.prepared_input_reuse_key
        .as_ref()
        .unwrap()
        .same_prepared_input_as(&input_key(selection, input), &mut |_| Ok(()))
        .unwrap()
}

/// A real completed boundary refuses reuse when its context is untracked.
pub fn untracked(row: RecordedOutput, context: DecisionContextUse) -> RecordedOutput {
    let boundary = CompletedHandlerFactBoundary::completed_for_test(1);
    let completed = boundary.seal_decision_reuse(decision(DECISION_KEY), context);
    assert!(completed.is_none());
    RecordedOutput {
        producing_implementation: None,
        required_publication: None,
        completed_handler_facts: Some(boundary),
        completed_decision_reuse: completed,
        prepared_input_reuse_key: None,
        ..row
    }
}
