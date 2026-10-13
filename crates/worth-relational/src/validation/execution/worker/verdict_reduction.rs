//! Reduction of one registered-rule evaluation into worker evidence.

use crate::validation::data::{InvariantCheckResult, InvariantVerdict, InvariantWitnessKey};

use super::super::envelope::InvariantWorkerResult;
use super::super::packets::InvariantWorkPacket;
use super::registered_rule::RegisteredInvariantEvaluation;
use crate::validation::engine::budget::InvariantBudget;

pub(super) struct ReducedInvariantEvaluation {
    pub(super) results: Vec<InvariantWorkerResult>,
    pub(super) diagnostic_observations: Vec<
        crate::authority::commit::preparation::diagnostics::observations::ValidationDiagnosticObservation,
    >,
}

pub(super) fn reduce_invariant_verdicts(
    packet: &InvariantWorkPacket<'_>,
    evaluation: RegisteredInvariantEvaluation,
    budget: Option<&dyn InvariantBudget>,
) -> Option<ReducedInvariantEvaluation> {
    let diagnostic_count = evaluation
        .verdicts
        .iter()
        .filter(|verdict| {
            !matches!(
                verdict,
                InvariantVerdict::Pass | InvariantVerdict::NotApplicable
            )
        })
        .count();
    let slots = evaluation.verdicts.len() * std::mem::size_of::<InvariantWorkerResult>()
        + diagnostic_count * std::mem::size_of::<crate::authority::commit::preparation::diagnostics::observations::ValidationDiagnosticObservation>();
    if budget.is_some_and(|budget| !budget.claim_result(slots as u64)) {
        return None;
    }
    let mut results = Vec::with_capacity(evaluation.verdicts.len());
    let mut diagnostic_observations = Vec::with_capacity(diagnostic_count);
    for verdict in evaluation.verdicts {
        let diagnostic = !matches!(
            verdict,
            InvariantVerdict::Pass | InvariantVerdict::NotApplicable
        );
        let witness = witness_for_verdict(&verdict, budget)?;
        let copies = if diagnostic { 3 } else { 2 };
        let clones = witness
            .owned_allocation_capacity_bytes()
            .saturating_mul(copies - 1)
            .saturating_add(
                super::super::envelope::rule_bytes(&evaluation.reported_rule)
                    .saturating_mul(copies),
            )
            .saturating_add(
                evaluation
                    .custom_provenance
                    .as_ref()
                    .map_or(0, |provenance| {
                        super::super::envelope::proposal_bytes(
                            provenance.proposal_identity.as_ref(),
                        )
                    }),
            );
        if budget.is_some_and(|budget| !budget.claim_result(clones)) {
            return None;
        }
        let result = InvariantCheckResult {
            execution_point: packet.registration.execution_point(),
            failure_effect: packet.registration.failure_effect(),
            rule: evaluation.reported_rule.clone(),
            witness: witness.clone(),
            groups: evaluation.groups,
            cost: evaluation.cost,
            custom_provenance: evaluation.custom_provenance.clone(),
            verdict,
        };
        let result_identity = result_identity(&result, witness);
        if !matches!(
            result.verdict,
            InvariantVerdict::Pass | InvariantVerdict::NotApplicable
        ) {
            diagnostic_observations.push(
                crate::authority::commit::preparation::diagnostics::observations::ValidationDiagnosticObservation {
                    packet_index: packet.packet_index,
                    result_identity: result_identity.clone(),
                },
            );
        }
        results.push(InvariantWorkerResult {
            result_identity,
            result,
        });
    }
    Some(ReducedInvariantEvaluation {
        results,
        diagnostic_observations,
    })
}

fn witness_for_verdict(
    verdict: &InvariantVerdict,
    budget: Option<&dyn InvariantBudget>,
) -> Option<InvariantWitnessKey> {
    match verdict {
        InvariantVerdict::Pass | InvariantVerdict::NotApplicable => {
            if budget.is_some_and(|budget| !budget.claim_result("pass".len() as u64)) {
                return None;
            }
            Some(InvariantWitnessKey::pass())
        }
        InvariantVerdict::Advisory { violation, .. } | InvariantVerdict::Violation(violation) => {
            violation.witness_key_checked(&mut |bytes| {
                budget.is_none_or(|budget| budget.claim_result(bytes))
            })
        }
    }
}

fn result_identity(
    result: &InvariantCheckResult,
    witness: InvariantWitnessKey,
) -> crate::authority::commit::preparation::reduction::identity::ValidationResultIdentity {
    crate::authority::commit::preparation::reduction::identity::ValidationResultIdentity::from_parts(
        result.execution_point,
        result.failure_effect,
        result.rule.clone(),
        witness,
    )
}
