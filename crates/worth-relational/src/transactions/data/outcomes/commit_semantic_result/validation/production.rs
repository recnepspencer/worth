use super::*;
use crate::symbols::data::StringInterner;
use crate::transactions::data::RelationalCommitSemanticResultRefusal as Refusal;
use crate::validation::engine::InvariantExecutionResult;

pub(super) mod references;

impl RelationalInvariantExecutionMeaning {
    pub(crate) fn from_original(
        original: &InvariantExecutionResult,
        symbols: &StringInterner,
    ) -> Result<Self, Refusal> {
        let metadata = original.metadata();
        let mut checks = Vec::with_capacity(original.results().len());
        for check in original.results() {
            let mut verdict = check.verdict.clone();
            references::canonicalize(&mut verdict, symbols)?;
            checks.push(RelationalInvariantCheckMeaning {
                execution_point: check.execution_point,
                failure_effect: check.failure_effect,
                rule: check.rule.clone(),
                witness: check.witness.as_str().to_owned(),
                groups: check.groups,
                declared_cost: check.cost,
                custom_provenance: check.custom_provenance.as_ref().map(|original| {
                    RelationalInvariantCustomProvenanceMeaning {
                        observation_kind: original.observation_kind,
                        version: original.version_id,
                        current_version: original.current_version_id,
                        touched: original.touched.clone(),
                        counts: original.counts,
                        non_applicability: original.non_applicability,
                    }
                }),
                verdict,
            });
        }
        let result = Self {
            metadata: RelationalInvariantMetadataMeaning {
                execution_point: metadata.execution_point(), observation_kind: metadata.observation_kind(),
                version: metadata.version_id(), current_version: metadata.current_version_id(),
                consumed_groups: metadata.consumed_groups(), applicable_groups: metadata.applicable_groups(),
                declared_cost: metadata.max_cost(), disposition: metadata.disposition(),
                plan_contract: metadata.plan_contract(), has_merged_plan: metadata.has_merged_plan(),
                preparation_strategy: metadata.preparation_strategy(),
                preparation_failures: metadata.preparation_failures().iter().map(|failure| {
                    use crate::authority::commit::preparation::diagnostics::failures::PreparationFailureClass as Original;
                    match failure {
                        Original::PlanningProofInsufficient => RelationalValidationPreparationFailure::PlanningProofInsufficient,
                        Original::ReductionIdentityConflict => RelationalValidationPreparationFailure::ReductionIdentityConflict,
                        Original::SerialStrategySelected => RelationalValidationPreparationFailure::SerialStrategySelected,
                        Original::PublicationIsolationViolation => RelationalValidationPreparationFailure::PublicationIsolationViolation,
                    }
                }).collect(),
                proof_scope: metadata.proof_boundary().map(|scope| RelationalInvariantProofScopeMeaning {
                    scope: scope.scope_class(), widened_causes: scope.widened_causes().to_vec(),
                }),
            }, checks,
            decisions: original.decision_log().iter().map(|decision| RelationalInvariantDecisionMeaning {
                execution_point: decision.execution_point, failure_effect: decision.failure_effect,
                rule: decision.rule.clone(), witness: decision.witness.as_str().to_owned(),
                decision: decision.decision, groups: decision.groups, declared_cost: decision.cost,
                custom_provenance_present: decision.custom_provenance_present,
            }).collect(),
        };
        if !result.is_consistent() {
            return Err(Refusal::ValidationMismatch);
        }
        Ok(result)
    }
}
