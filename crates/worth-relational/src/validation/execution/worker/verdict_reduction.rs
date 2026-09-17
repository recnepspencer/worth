//! Reduction of one registered-rule evaluation into worker evidence.

use crate::validation::data::{InvariantCheckResult, InvariantVerdict, InvariantWitnessKey};

use super::super::envelope::InvariantWorkerResult;
use super::super::packets::InvariantWorkPacket;
use super::registered_rule::RegisteredInvariantEvaluation;

pub(super) struct ReducedInvariantEvaluation {
    pub(super) results: Vec<InvariantWorkerResult>,
    pub(super) diagnostic_observations: Vec<
        crate::authority::commit::preparation::diagnostics::observations::ValidationDiagnosticObservation,
    >,
}

pub(super) fn reduce_invariant_verdicts(
    packet: &InvariantWorkPacket<'_>,
    evaluation: RegisteredInvariantEvaluation,
) -> ReducedInvariantEvaluation {
    let mut results = Vec::with_capacity(evaluation.verdicts.len());
    let mut diagnostic_observations = Vec::new();
    for verdict in evaluation.verdicts {
        let witness = witness_for_verdict(&verdict);
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
    ReducedInvariantEvaluation {
        results,
        diagnostic_observations,
    }
}

fn witness_for_verdict(verdict: &InvariantVerdict) -> InvariantWitnessKey {
    match verdict {
        InvariantVerdict::Pass | InvariantVerdict::NotApplicable => InvariantWitnessKey::pass(),
        InvariantVerdict::Advisory { violation, .. } | InvariantVerdict::Violation(violation) => {
            violation.witness_key()
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
