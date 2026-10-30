//! A private fresh Signal partition binds one exact baseline/candidate comparison.
use super::{
    BridgeConditionalCausalObservation, BridgeConditionalCondition, BridgeConditionalDenial,
    BridgeConditionalDenialKind, BridgeConditionalEvaluationAdmissionRequest,
    BridgeConditionalExecutionRequest, BridgeConditionalSignalBasisBinding,
    BridgeOwnedSignalRuntime,
};
use worth_signal::facade::{SignalConditionalDecisionClass, SignalConditionalVersionComparator};

/// Descriptive custody of two actual computed effects in one private evaluation slot.
/// This grants no execution, publication, Native-field or disclosure authority.
#[derive(Clone, Debug)]
pub struct BridgeExactOutputComparison {
    baseline: BridgeConditionalCausalObservation,
    candidate: BridgeConditionalCausalObservation,
}

impl BridgeExactOutputComparison {
    pub fn baseline(&self) -> &BridgeConditionalCausalObservation {
        &self.baseline
    }
    pub fn candidate(&self) -> &BridgeConditionalCausalObservation {
        &self.candidate
    }
    pub fn same_comparison(&self, other: &Self) -> bool {
        self.baseline.same_observation(&other.baseline)
            && self.candidate.same_observation(&other.candidate)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeOutputComparisonStage {
    Admission,
    Baseline,
    Candidate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeOutputComparisonDenial {
    stage: BridgeOutputComparisonStage,
    cause: BridgeConditionalDenial,
}
impl BridgeOutputComparisonDenial {
    pub fn stage(&self) -> BridgeOutputComparisonStage {
        self.stage
    }
    pub fn cause(&self) -> &BridgeConditionalDenial {
        &self.cause
    }
    pub fn into_cause(self) -> BridgeConditionalDenial {
        self.cause
    }
}

impl BridgeOwnedSignalRuntime {
    /// Computes an exact baseline, then candidate, in a fresh session which never escapes.
    /// The compute provider must return each supplied u64 as its output version.
    /// Partial failure drops the private slot and never returns a comparison.
    pub fn compare_exact_output(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        baseline_version: u64,
        candidate_version: u64,
    ) -> Result<BridgeExactOutputComparison, BridgeOutputComparisonDenial> {
        use BridgeOutputComparisonStage as Stage;
        let refuse = |stage, kind, detail| BridgeOutputComparisonDenial {
            stage,
            cause: BridgeConditionalDenial::new(kind, detail),
        };
        if request.attempt < 2 {
            return Err(refuse(
                Stage::Admission,
                BridgeConditionalDenialKind::AttemptMismatch,
                "output comparison requires distinct nonzero baseline and candidate attempts",
            ));
        }
        if request.bridge_snapshot_identity.is_some()
            || !request.lowering.correspondences.is_empty()
        {
            return Err(refuse(
                Stage::Admission,
                BridgeConditionalDenialKind::SourcePostureMismatch,
                "output comparison uses a private source-free computation slot",
            ));
        }
        if request.lowering.contract().condition() != &BridgeConditionalCondition::OnDemand
            || request.lowering.contract().output_comparator()
                != SignalConditionalVersionComparator::Exact
        {
            return Err(refuse(
                Stage::Admission,
                BridgeConditionalDenialKind::SignalContractMismatch,
                "output comparison requires installed on-demand exact output equivalence",
            ));
        }
        let session = self
            .admit_conditional_evaluation(
                BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                    signal_basis,
                ),
                execution,
            )
            .map_err(|cause| BridgeOutputComparisonDenial {
                stage: Stage::Admission,
                cause,
            })?;
        // admit_evaluation mints a unique non-Clone admission/partition. This
        // session is never exposed to a caller or provider, so another execution
        // cannot replace this baseline between the two ordinary owner calls.
        let baseline_request = BridgeConditionalExecutionRequest {
            lowering: request.lowering,
            query_binding_identity: request.query_binding_identity,
            query_capability_identity: request.query_capability_identity,
            snapshot_identity: request.snapshot_identity,
            truth_branch_identity: request.truth_branch_identity,
            bridge_snapshot_identity: None,
            execution_identity: request.execution_identity,
            attempt: request.attempt - 1,
        };
        let mut before = baseline_version;
        let baseline = self
            .execute_admitted_conditional(execution, &session, baseline_request, &mut before)
            .map_err(|cause| BridgeOutputComparisonDenial {
                stage: Stage::Baseline,
                cause,
            })?;
        if !computed(&baseline) || baseline.signal().output_version() != baseline_version {
            return Err(refuse(
                Stage::Baseline,
                BridgeConditionalDenialKind::SignalContractMismatch,
                "computed baseline did not establish the exact supplied output version",
            ));
        }
        let mut after = candidate_version;
        let candidate = self
            .execute_admitted_conditional(execution, &session, request, &mut after)
            .map_err(|cause| BridgeOutputComparisonDenial {
                stage: Stage::Candidate,
                cause,
            })?;
        if !computed(&candidate)
            || candidate.signal().output_aspect() != baseline.signal().output_aspect()
            || candidate.signal().output_version_before() != baseline.signal().output_version()
            || candidate.signal().output_version() != candidate_version
        {
            return Err(refuse(
                Stage::Candidate,
                BridgeConditionalDenialKind::SignalContractMismatch,
                "computed candidate did not retain this exact baseline and supplied output version",
            ));
        }
        Ok(BridgeExactOutputComparison {
            baseline: baseline.retain_causal_observation(),
            candidate: candidate.retain_causal_observation(),
        })
    }
}

fn computed(evidence: &super::BridgeConditionalDecisionEvidence) -> bool {
    evidence.signal().computed_evaluation().is_some()
        && matches!(
            evidence.signal().class(),
            SignalConditionalDecisionClass::ComputedChanged
                | SignalConditionalDecisionClass::ComputedRevertedClean
        )
}
