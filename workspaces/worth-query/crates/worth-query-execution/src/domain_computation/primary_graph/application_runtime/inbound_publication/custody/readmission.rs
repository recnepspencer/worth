//! Readmission retains a genuine World recovery handle with its recorded transport evidence.
use super::super::recovered::RecordedCompletionContinuation;
use super::*;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryInboundAdmissionDenial as Denial;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph::application_runtime) fn readmit_unpublished_transport_completion(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
        attempt: &[u8; 32],
        observation: &[u8; 32],
        continuation: RecordedCompletionContinuation,
    ) -> Result<(), Denial> {
        let mut custody = self
            .runtime
            .transport_completion_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let entry = custody
            .entries
            .get_mut(correlation)
            .ok_or(Denial::UnknownCorrelation)?;
        match &entry.state {
            RetainedCompletion::Ready(evidence) => {
                if evidence.attempt_identity().bytes() != attempt
                    || evidence.observation_identity().map(|id| id.bytes()) != Some(observation)
                {
                    return Err(Denial::IncompatibleMeaning);
                }
                entry.state = RetainedCompletion::Recorded(Arc::clone(evidence), continuation);
                Ok(())
            }
            RetainedCompletion::Recorded(_, existing) if existing.same_original(&continuation) => {
                Ok(())
            }
            RetainedCompletion::Recorded(_, _)
            | RetainedCompletion::Publishing
            | RetainedCompletion::PerformedPending(_, _)
            | RetainedCompletion::Unpublished(_, _, _)
            | RetainedCompletion::Recovering
            | RetainedCompletion::StaleRelease(_, _) => Err(Denial::RecoveryUnavailable),
        }
    }
}
