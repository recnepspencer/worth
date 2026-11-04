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
    PreparedDecisionReuseContext::new_for_test(
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
    PreparedInputReuseKey::new_for_test(
        selection.clone(),
        input,
        InstalledProducerEdition::for_test_for_test([0x51; 32]),
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
        .seal_decision_reuse_for_test(
            decision(DECISION_KEY),
            DecisionContextUse::default().key_for_test(),
        )
        .unwrap();
    {
        let __producing_implementation = None;
        let __required_publication = None;
        let __native_prior_checkpoint = None;
        let __computation_source = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false);
        let __consumed_outputs = Arc::clone(consumed);
        let __completed_handler_facts = Some(boundary);
        let __completed_decision_reuse = Some(completed);
        let __prepared_input_reuse_key = Some(input_key(selection, INPUT));
        let __native_output_witness = OnceLock::from(Arc::clone(witness));
        let mut __base = row;
        *__base.producing_implementation_mut_for_test() = __producing_implementation;
        *__base.required_publication_mut_for_test() = __required_publication;
        *__base.native_prior_checkpoint_mut_for_test() = __native_prior_checkpoint;
        *__base.computation_source_mut_for_test() = __computation_source;
        *__base.consumed_outputs_mut_for_test() = __consumed_outputs;
        *__base.completed_handler_facts_mut_for_test() = __completed_handler_facts;
        *__base.completed_decision_reuse_mut_for_test() = __completed_decision_reuse;
        *__base.prepared_input_reuse_key_mut_for_test() = __prepared_input_reuse_key;
        *__base.native_output_witness_mut_for_test() = __native_output_witness;
        __base
    }
}

pub fn assert_continues_the_performed_read(row: &RecordedOutput) {
    assert!((*row.performed_origin_for_test()).is_none());
    assert_eq!(row.verification_requirement_for_test(), None);
    assert_eq!(
        (*row.completed_handler_facts_for_test())
            .as_ref()
            .unwrap()
            .handler_fact_count_for_test(),
        1
    );
    let completed = (*row.completed_decision_reuse_for_test()).as_ref().unwrap();
    let same = |key| decision(key).matches_completed_for_test(completed);
    assert!(same(DECISION_KEY));
    assert!(!same([0x72; 32]));
}

pub fn continues_input(
    row: &RecordedOutput,
    selection: &WorthQueryObservedSourceSelection,
    input: [u8; 32],
) -> bool {
    (*row.prepared_input_reuse_key_for_test())
        .as_ref()
        .unwrap()
        .same_prepared_input_as_for_test(&input_key(selection, input), &mut |_| Ok(()))
        .unwrap()
}

/// A real completed boundary refuses reuse when its context is untracked.
pub fn untracked(row: RecordedOutput, context: DecisionContextUse) -> RecordedOutput {
    let boundary = CompletedHandlerFactBoundary::completed_for_test(1);
    let completed = boundary.seal_decision_reuse_for_test(decision(DECISION_KEY), context);
    assert!(completed.is_none());
    {
        let __producing_implementation = None;
        let __required_publication = None;
        let __completed_handler_facts = Some(boundary);
        let __completed_decision_reuse = completed;
        let __prepared_input_reuse_key = None;
        let mut __base = row;
        *__base.producing_implementation_mut_for_test() = __producing_implementation;
        *__base.required_publication_mut_for_test() = __required_publication;
        *__base.completed_handler_facts_mut_for_test() = __completed_handler_facts;
        *__base.completed_decision_reuse_mut_for_test() = __completed_decision_reuse;
        *__base.prepared_input_reuse_key_mut_for_test() = __prepared_input_reuse_key;
        __base
    }
}
