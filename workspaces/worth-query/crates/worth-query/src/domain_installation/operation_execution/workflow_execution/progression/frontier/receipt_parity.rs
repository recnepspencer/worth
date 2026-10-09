use super::{
    domain, foundation, inputs, member_by_member_frontier, phased_workflow, start, PhaseDomain,
    PhaseFamily, PhaseOperation,
};

#[test]
fn full_receipt_evidence_matches_reference_on_success_and_failure() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    for mode in 0..=5 {
        for position in 0..3 {
            let mut reference_workspace =
                phased_workflow::reference_workspace("phase-full-evidence");
            let mut phased_workspace = phased_workflow::phased_workspace("phase-full-evidence");
            let members = || {
                let mut members = inputs();
                members[position].1 =
                    domain::WorthQueryWorkflowValue::Text(format!("{}:{mode}", 7 + position));
                members
            };
            let reference_outcome = member_by_member_frontier(
                start(&mut reference_workspace),
                members(),
                &mut reference_workspace,
            );
            let phased = start(&mut phased_workspace)
                .advance_admitted_frontier(members(), &mut phased_workspace);
            assert_eq!(
                std::mem::discriminant(&phased),
                std::mem::discriminant(&reference_outcome)
            );
            match (&reference_outcome, &phased) {
                (
                    worth_proof::TransitionOutcome::Success(reference_outcome),
                    worth_proof::TransitionOutcome::Success(phased),
                ) => assert_eq!(
                    super::owner_counters(phased.counters()),
                    super::owner_counters(reference_outcome.counters())
                ),
                (
                    worth_proof::TransitionOutcome::Failed(reference_outcome),
                    worth_proof::TransitionOutcome::Failed(phased),
                )
                | (
                    worth_proof::TransitionOutcome::Denied(reference_outcome),
                    worth_proof::TransitionOutcome::Denied(phased),
                ) => {
                    assert_eq!(phased.kind(), reference_outcome.kind());
                    assert_eq!(
                        super::owner_counters(phased.counters()),
                        super::owner_counters(reference_outcome.counters())
                    );
                    assert_eq!(
                        phased.executed_effects().len(),
                        reference_outcome.executed_effects().len()
                    );
                    for (actual, expected) in phased
                        .executed_effects()
                        .iter()
                        .zip(reference_outcome.executed_effects())
                    {
                        assert!(actual.semantic_replay_eq(expected));
                    }
                }
                _ => panic!("outcome kinds must agree"),
            }
            let (expected, actual) = (receipts(&reference_outcome), receipts(&phased));
            assert_eq!(actual.len(), expected.len());
            for (expected, actual) in expected.iter().zip(actual) {
                assert_eq!(actual.stage_identity(), expected.stage_identity());
                assert_eq!(actual.input(), expected.input());
                assert_eq!(actual.output_semantics(), expected.output_semantics());
                assert_eq!(actual.result_state(), expected.result_state());
                assert_eq!(actual.warnings(), expected.warnings());
                assert_eq!(
                    super::owner_counters(actual.counters()),
                    super::owner_counters(expected.counters())
                );
                assert_eq!(
                    actual.predecessor_stage_identities(),
                    expected.predecessor_stage_identities()
                );
                assert_eq!(
                    actual.graph_receipts().len(),
                    expected.graph_receipts().len()
                );
                assert_eq!(
                    actual.primary_read_evidence().len(),
                    expected.primary_read_evidence().len()
                );
                for (actual, expected) in actual
                    .primary_read_evidence()
                    .iter()
                    .zip(expected.primary_read_evidence())
                {
                    assert!(actual.semantic_replay_eq(expected));
                }
                assert_eq!(
                    actual.effect_evidence().len(),
                    expected.effect_evidence().len()
                );
                for (actual, expected) in actual
                    .effect_evidence()
                    .iter()
                    .zip(expected.effect_evidence())
                {
                    assert!(actual.semantic_replay_eq(expected));
                }
                assert_eq!(actual.invariant_outcomes(), expected.invariant_outcomes());
                assert_eq!(
                    actual
                        .parallel_admission()
                        .map(|admission| admission.lower_receipt()),
                    expected
                        .parallel_admission()
                        .map(|admission| admission.lower_receipt())
                );
                assert_eq!(
                    actual.conditional_provenance().len(),
                    expected.conditional_provenance().len()
                );
                assert!(actual.domain_evidence().is_none() && expected.domain_evidence().is_none());
            }
        }
    }
}
fn receipts(
    outcome: &super::super::super::super::workflow_progression::WorthQueryWorkflowAdvanceOutcome<
        PhaseDomain,
        PhaseOperation,
        PhaseFamily,
        foundation::MutationPreparationLaneWitness,
    >,
) -> &[domain::WorthQueryWorkflowStageReceipt] {
    match outcome {
        worth_proof::TransitionOutcome::Success(run) => run.receipts(),
        worth_proof::TransitionOutcome::Failed(denial)
        | worth_proof::TransitionOutcome::Denied(denial) => denial.completed_stage_receipts(),
        _ => panic!("unexpected fixture outcome"),
    }
}
