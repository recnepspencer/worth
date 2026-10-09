use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, runtime};

use super::{member_by_member_frontier, phased_workflow};
use phased_workflow::{PhaseDomain, PhaseFamily, PhaseOperation};

type Run = domain::WorthQueryWorkflowRun<
    PhaseDomain,
    PhaseOperation,
    PhaseFamily,
    worth_query::facade::foundation::MutationPreparationLaneWitness,
>;

#[derive(Debug, PartialEq)]
pub(super) struct Record {
    pub(super) outcome: Option<domain::WorthQueryWorkflowAdvanceDenialKind>,
    counters: domain::WorthQueryWorkflowRunCounters,
    pub(super) receipts: Vec<(
        String,
        Vec<String>,
        domain::WorthQueryWorkflowSemanticValue,
        domain::WorthQueryWorkflowRunCounters,
    )>,
    pub(super) effects: Vec<(String, String)>,
    reads: Vec<(String, String)>,
    outputs: Vec<domain::WorthQueryWorkflowSemanticValue>,
    result_states: Vec<Option<domain::WorthQueryOperationResultState>>,
    warnings: Vec<Vec<domain::WorthQueryWorkflowStageWarning>>,
    invariants: Vec<Vec<domain::WorthQueryWorkflowInvariantOutcome>>,
}

fn start(workspace: &mut runtime::WorthQueryWorkspace) -> Run {
    let installed = workspace.domain(PhaseDomain).unwrap();
    workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(PhaseFamily)
        .bind(&installed, PhaseOperation)
        .unwrap()
        .admit_workflow_resources(
            phased_workflow::world::execution_resource_request(),
            workspace,
        )
        .unwrap()
        .start_workflow(workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            workspace,
        )
        .unwrap()
}

fn run(
    workspace: &mut runtime::WorthQueryWorkspace,
    seed: u64,
    failure: Option<(usize, u64)>,
    member_by_member: bool,
) -> Record {
    let members = phased_workflow::MEMBERS
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let mode = failure
                .filter(|(at, _)| *at == index)
                .map_or(0, |(_, mode)| mode);
            (
                name.to_string(),
                domain::WorthQueryWorkflowValue::Text(format!("{}:{mode}", seed + index as u64)),
            )
        })
        .collect::<Vec<_>>();
    let run = start(workspace);
    // The reference completes each singleton before preparing the next; the
    // subject prepares and computes the complete frontier before any apply.
    let outcome = if member_by_member {
        member_by_member_frontier(run, members, workspace)
    } else {
        run.advance_admitted_frontier(members, workspace)
    };
    match outcome {
        TransitionOutcome::Success(run) => record(None, run.counters(), run.receipts(), &[]),
        TransitionOutcome::Failed(denial) | TransitionOutcome::Denied(denial) => record(
            Some(denial.kind().clone()),
            denial.counters(),
            denial.completed_stage_receipts(),
            denial.executed_effects(),
        ),
        _ => panic!("unexpected fixture outcome"),
    }
}

pub(super) fn record(
    outcome: Option<domain::WorthQueryWorkflowAdvanceDenialKind>,
    counters: domain::WorthQueryWorkflowRunCounters,
    receipts: &[domain::WorthQueryWorkflowStageReceipt],
    stopped_effects: &[domain::WorthQueryWorkflowEffectEvidence],
) -> Record {
    let effects = if outcome.is_some() {
        stopped_effects.to_vec()
    } else {
        receipts
            .iter()
            .flat_map(|receipt| receipt.effect_evidence().iter().cloned())
            .collect()
    };
    Record {
        outcome,
        outputs: receipts
            .iter()
            .map(|receipt| receipt.output_semantics().clone())
            .collect(),
        result_states: receipts
            .iter()
            .map(|receipt| receipt.result_state())
            .collect(),
        warnings: receipts
            .iter()
            .map(|receipt| receipt.warnings().to_vec())
            .collect(),
        invariants: receipts
            .iter()
            .map(|receipt| receipt.invariant_outcomes().to_vec())
            .collect(),
        counters: super::owner_counters(counters),
        receipts: receipts
            .iter()
            .map(|receipt| {
                (
                    receipt.stage_identity().into(),
                    receipt.predecessor_stage_identities().to_vec(),
                    receipt.input().clone(),
                    super::owner_counters(receipt.counters()),
                )
            })
            .collect(),
        effects: effects
            .iter()
            .map(|effect| {
                let mutation = effect.mutation_receipt().unwrap();
                (
                    mutation
                        .target_collection_identity()
                        .unwrap()
                        .as_str()
                        .into(),
                    mutation.declared_aspect_value_digest().unwrap().into(),
                )
            })
            .collect(),
        reads: receipts
            .iter()
            .flat_map(|receipt| {
                receipt.primary_read_evidence().iter().map(|read| {
                    (
                        receipt.stage_identity().into(),
                        read.read_receipt().canonical_query_digest().into(),
                    )
                })
            })
            .collect(),
    }
}

// Width comparisons keep all billing; only the historical owner oracle omits
// the new authority measurement that did not exist in part two.
pub(super) fn record_billed(
    outcome: Option<domain::WorthQueryWorkflowAdvanceDenialKind>,
    counters: domain::WorthQueryWorkflowRunCounters,
    receipts: &[domain::WorthQueryWorkflowStageReceipt],
    stopped_effects: &[domain::WorthQueryWorkflowEffectEvidence],
) -> Record {
    let mut record = record(outcome, counters, receipts, stopped_effects);
    record.counters = counters;
    for (observed, receipt) in record.receipts.iter_mut().zip(receipts) {
        observed.3 = receipt.counters();
    }
    record
}

#[test]
fn member_by_member_frontiers_preserve_success_failure_denial_and_partial_effects() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    for seed in [1, 7, 29] {
        let mut workspace = phased_workflow::reference_workspace("reference-fold-success");
        let success = run(&mut workspace, seed, None, true);
        assert_eq!(success.outcome, None);
        assert_eq!(success.effects.len(), phased_workflow::MEMBERS.len());
        for position in 0..phased_workflow::MEMBERS.len() {
            for mode in 1..=5 {
                let mut workspace = phased_workflow::reference_workspace("reference-fold-failure");
                let failed = run(&mut workspace, seed, Some((position, mode)), true);
                assert!(failed.outcome.is_some());
                assert_eq!(failed.receipts.len(), position + 1);
                assert_eq!(failed.effects.len(), position + usize::from(mode >= 4));
            }
        }
    }
}

#[test]
fn phased_frontiers_match_member_by_member_at_every_failure_position() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    for seed in [1, 7, 29] {
        for failure in std::iter::once(None)
            .chain((0..3).flat_map(|position| (1..=5).map(move |mode| Some((position, mode)))))
        {
            let mut reference_workspace =
                phased_workflow::reference_workspace("phase-differential");
            let mut phased = phased_workflow::phased_workspace("phase-differential");
            _probe.take_computes();
            let expected = run(&mut reference_workspace, seed, failure, true);
            let actual = run(&mut phased, seed, failure, false);
            assert_eq!(actual, expected, "seed={seed} failure={failure:?}");
            // Preparation stops at its failure; otherwise all three members
            // compute once, including the suffix whose results must be dropped.
            let prepared = failure
                .filter(|(_, mode)| *mode == 1)
                .map_or(3, |(position, _)| position);
            assert_eq!(
                _probe.take_computes(),
                phased_workflow::MEMBERS[..prepared]
                    .iter()
                    .map(|stage| (stage.to_string(), 17))
                    .collect::<Vec<_>>()
            );
        }
    }
}
#[test]
fn idempotent_retry_preserves_reference_failure_and_counters() {
    let _probe = phased_workflow::fold_executor::ComputeProbe::new();
    fn attempts(workspace: &mut runtime::WorthQueryWorkspace) -> Vec<Record> {
        let attempt = start(workspace)
            .prepare_stage_attempt(
                "left",
                domain::WorthQueryWorkflowIntentValue::Text("7:3".into()),
            )
            .unwrap();
        let first_identity = attempt.identity().to_owned();
        let failure = match attempt.execute(workspace) {
            domain::WorthQueryWorkflowStageAttemptOutcome::Retryable(failure) => failure,
            _ => panic!("effect-free failure must be retryable"),
        };
        let first = record(
            Some(failure.denial().kind().clone()),
            failure.denial().counters(),
            failure.denial().completed_stage_receipts(),
            failure.denial().executed_effects(),
        );
        let retry = failure.retry();
        assert_ne!(retry.identity(), first_identity);
        let failure = match retry.execute(workspace) {
            domain::WorthQueryWorkflowStageAttemptOutcome::Retryable(failure) => failure,
            _ => panic!("retry must fail without publishing effects"),
        };
        vec![
            first,
            record(
                Some(failure.denial().kind().clone()),
                failure.denial().counters(),
                failure.denial().completed_stage_receipts(),
                failure.denial().executed_effects(),
            ),
        ]
    }
    let mut reference_workspace = phased_workflow::reference_workspace("phase-retry");
    let mut phased = phased_workflow::phased_workspace("phase-retry");
    assert_eq!(attempts(&mut phased), attempts(&mut reference_workspace));
}

impl Record {
    pub(super) fn assert_owner_prefix_of(&self, serial: &Self) {
        let count = self.receipts.len();
        for (actual, expected) in self.receipts.iter().zip(&serial.receipts) {
            assert_eq!(
                (
                    &actual.0,
                    &actual.1,
                    &actual.2,
                    super::owner_counters(actual.3)
                ),
                (
                    &expected.0,
                    &expected.1,
                    &expected.2,
                    super::owner_counters(expected.3)
                )
            );
        }
        assert_eq!(self.outputs, serial.outputs[..count]);
        assert_eq!(self.result_states, serial.result_states[..count]);
        assert_eq!(self.warnings, serial.warnings[..count]);
        assert_eq!(self.invariants, serial.invariants[..count]);
        assert_eq!(self.reads, serial.reads[..self.reads.len()]);
        assert_eq!(self.effects, serial.effects[..self.effects.len()]);
    }
}
