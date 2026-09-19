use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionResolver, SignalConditionalDecisionEvidence,
    SignalConditionalExecutionFailure, SignalConditionalExecutionRequest,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output::NodeEvaluationResult;
use crate::data::proof::SignalInvalidationExecutionReceipt;
use crate::logic::transaction::{SignalObservationAdmissionDenial, SignalObservationRequest};

use super::draft::ConditionalEvaluationDraft;
use super::{
    SignalEvaluationPartition, SignalPartitionConditionalUnwind,
    SignalPartitionConditionalUnwindReason, SignalRejectedConditionalEvaluation,
};

mod completion;

#[derive(Debug)]
pub(crate) enum SignalPartitionConditionalDenial {
    Activation(SignalError),
    StorageAdmission(SignalError),
    ObservationAdmission(SignalObservationAdmissionDenial),
    UnconsumedUnwind,
}

/// Preserve execution failure and observation completion independently. A
/// failed compute does not erase work already observed by the Signal owner.
pub(crate) struct SignalPartitionConditionalCompletion {
    decision: Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>,
    observation: Result<Option<SignalInvalidationExecutionReceipt>, SignalError>,
    rejected: Option<SignalRejectedConditionalEvaluation>,
}

enum ActivatedConditionalOutcome {
    Completed(SignalPartitionConditionalCompletion),
    Unwound {
        payload: Box<dyn std::any::Any + Send>,
        report: SignalPartitionConditionalUnwindReason,
    },
}

impl SignalPartitionConditionalCompletion {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>,
        Result<Option<SignalInvalidationExecutionReceipt>, SignalError>,
        Option<SignalRejectedConditionalEvaluation>,
    ) {
        (self.decision, self.observation, self.rejected)
    }
}

impl SignalEvaluationPartition {
    /// Named kernel operation used after service admission. Provider callbacks
    /// receive their existing condition/compute contracts, never mutable graph
    /// access. Observation custody ends before storage deactivation.
    pub(crate) fn execute_conditional(
        &mut self,
        graph: &mut SignalGraph,
        request: SignalConditionalExecutionRequest<'_>,
        condition: &mut impl InstalledSignalConditionResolver,
        comparator: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
    ) -> Result<SignalPartitionConditionalCompletion, SignalPartitionConditionalDenial> {
        if self.pending_unwind.is_some() {
            return Err(SignalPartitionConditionalDenial::UnconsumedUnwind);
        }
        // One selected-definition allowance spans admission, execution and
        // finalization; no nested stage may reset this context.
        let mut work = crate::data::retained_storage::RetainedStoragePreparation::new(
            self.definitions
                .installed_policy()
                .conditional_evaluation_budget()
                .maximum_attempt_visits,
        );
        let draft = ConditionalEvaluationDraft::begin(self, &mut work)
            .map_err(SignalPartitionConditionalDenial::StorageAdmission)?;
        let outcome = draft
            .partition
            .execute(
                graph,
                |graph| -> Result<_, SignalPartitionConditionalDenial> {
                    let observation = graph
                        .begin_observation_session(SignalObservationRequest::operation())
                        .map_err(SignalPartitionConditionalDenial::ObservationAdmission)?;
                    let attempt = graph.execute_installed_conditional_attempt(
                        request, condition, comparator, compute, &mut work,
                    );
                    Ok(completion::finish_attempt(
                        graph,
                        observation,
                        attempt,
                        &mut work,
                    ))
                },
            )
            .map_err(SignalPartitionConditionalDenial::Activation)??;
        match outcome {
            ActivatedConditionalOutcome::Completed(mut completion) => {
                if completion.decision.is_ok() && completion.observation.is_ok() {
                    draft.install();
                } else {
                    completion.rejected = Some(draft.reject());
                }
                Ok(completion)
            }
            ActivatedConditionalOutcome::Unwound { payload, report } => {
                let rejected = draft.reject();
                self.pending_unwind = Some(SignalPartitionConditionalUnwind::new(report, rejected));
                std::panic::resume_unwind(payload)
            }
        }
    }
}
