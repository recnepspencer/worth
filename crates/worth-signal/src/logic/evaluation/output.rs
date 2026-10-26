use crate::data::aspect::AspectVersion;
use crate::data::output::{IntoNodeEvaluationResult, NodeEvaluationResult};
use crate::data::temporal::LoweredTemporalEligibility;
use crate::data::trace::CausalityMetadata;
use crate::logic::prepared::PreparedDependencyCapture;
use crate::logic::prepared::{PreparedEvaluation, PreparedEvaluationOutcome, PreparedTraceData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationOutput {
    pub(crate) result: NodeEvaluationResult,
    pub(crate) trace_data: PreparedTraceData,
    pub(crate) outcome: PreparedEvaluationOutcome,
}

impl EvaluationOutput {
    pub fn from_result(result: impl IntoNodeEvaluationResult) -> Self {
        Self {
            result: result.into_evaluation_result(),
            trace_data: PreparedTraceData::default(),
            outcome: PreparedEvaluationOutcome::Evaluate,
        }
    }

    pub fn validated_clean() -> Self {
        Self {
            result: NodeEvaluationResult::from_version(AspectVersion::zero()),
            trace_data: PreparedTraceData::default(),
            outcome: PreparedEvaluationOutcome::ValidatedClean,
        }
    }

    pub fn deferred_by_condition() -> Self {
        Self {
            result: NodeEvaluationResult::from_version(AspectVersion::zero()),
            trace_data: PreparedTraceData::default(),
            outcome: PreparedEvaluationOutcome::DeferredByCondition,
        }
    }

    pub fn deferred_by_time(temporal_eligibility: LoweredTemporalEligibility) -> Self {
        let mut output = Self::deferred_by_condition();
        output.trace_data.temporal_eligibility = Some(temporal_eligibility);
        output
    }

    pub fn reverted_clean_by_condition() -> Self {
        Self {
            result: NodeEvaluationResult::from_version(AspectVersion::zero()),
            trace_data: PreparedTraceData::default(),
            outcome: PreparedEvaluationOutcome::RevertedCleanByCondition,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.trace_data.labels.push(label.into());
        self
    }

    pub fn with_labels(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.trace_data
            .labels
            .extend(labels.into_iter().map(Into::into));
        self
    }

    pub fn with_causality(mut self, causality: CausalityMetadata) -> Self {
        self.trace_data.causality = Some(causality);
        self
    }

    pub fn with_temporal_eligibility(
        mut self,
        temporal_eligibility: LoweredTemporalEligibility,
    ) -> Self {
        self.trace_data.temporal_eligibility = Some(temporal_eligibility);
        self
    }

    pub(crate) fn into_prepared(
        self,
        dependencies: PreparedDependencyCapture,
    ) -> PreparedEvaluation {
        PreparedEvaluation {
            result: self.result,
            dependencies,
            trace_data: self.trace_data,
            outcome: self.outcome,
            origin: crate::logic::prepared::PreparedEvaluationOrigin::DirectPrecompute,
            memo_decision: crate::logic::prepared::PreparedMemoDecision::None,
            keyed: None,
        }
    }
}

pub trait IntoEvaluationOutput {
    fn into_evaluation_output(self) -> EvaluationOutput;

    /// A declared upper bound for output and trace heap, excluding input capture.
    fn maximum_result_heap_bytes() -> Option<u64>
    where
        Self: Sized,
    {
        None
    }
}

impl IntoEvaluationOutput for EvaluationOutput {
    fn into_evaluation_output(self) -> EvaluationOutput {
        self
    }
}

impl<T> IntoEvaluationOutput for T
where
    T: IntoNodeEvaluationResult,
{
    fn into_evaluation_output(self) -> EvaluationOutput {
        EvaluationOutput::from_result(self)
    }

    fn maximum_result_heap_bytes() -> Option<u64> {
        T::maximum_result_heap_bytes()
    }
}
