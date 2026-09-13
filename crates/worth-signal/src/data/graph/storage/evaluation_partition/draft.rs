use super::{SignalEvaluationPartition, SignalEvaluationStorage};

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod compaction_tests;
#[cfg(test)]
mod tests;

/// Rejected derived state retained with its performed diagnostics. This value
/// grants no execution authority and has no route back into a canonical slot.
pub(crate) struct SignalRejectedConditionalEvaluation {
    storage: SignalEvaluationStorage,
}

/// One candidate for the whole conditional attempt, including preparation.
/// Observation and traversal remain with the existing partition. Resource
/// admission must cover both roots before this ordinary persistent fork runs.
pub(in crate::data::graph) struct ConditionalEvaluationDraft<'a> {
    pub(in crate::data::graph) partition: &'a mut SignalEvaluationPartition,
    previous: Option<SignalEvaluationStorage>,
}

impl<'a> ConditionalEvaluationDraft<'a> {
    pub(in crate::data::graph) fn begin(
        partition: &'a mut SignalEvaluationPartition,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<Self, crate::data::error::SignalError> {
        let candidate = partition.evaluation.try_fork_persistent(work)?;
        let previous = std::mem::replace(&mut partition.evaluation, candidate);
        Ok(Self {
            partition,
            previous: Some(previous),
        })
    }

    /// Called only after activation has restored the ambient graph.
    pub(in crate::data::graph) fn install(mut self) {
        drop(self.previous.take());
    }

    /// Restore canonical storage before handing out rejected-state custody.
    pub(in crate::data::graph) fn reject(mut self) -> SignalRejectedConditionalEvaluation {
        let mut previous = self.previous.take().expect("draft owns previous roots");
        previous.preserve_issuance_from(&self.partition.evaluation);
        let storage = std::mem::replace(&mut self.partition.evaluation, previous);
        SignalRejectedConditionalEvaluation { storage }
    }
}

impl Drop for ConditionalEvaluationDraft<'_> {
    fn drop(&mut self) {
        if let Some(mut previous) = self.previous.take() {
            // An activation/admission failure has no acquired attempt report.
            // Restore first, including when unwinding before session admission.
            previous.preserve_issuance_from(&self.partition.evaluation);
            let discarded = std::mem::replace(&mut self.partition.evaluation, previous);
            drop(discarded);
        }
    }
}

#[cfg(test)]
impl SignalRejectedConditionalEvaluation {
    pub(crate) fn diagnostics_for_test(&self) -> serde_json::Value {
        self.storage.diagnostics_for_test()
    }

    pub(crate) fn retained_artifact_for_test(
        &self,
        node: crate::data::handle::NodeId,
    ) -> Option<&crate::data::trace::RetainedDiagnosticArtifact> {
        self.storage.retained_artifact_for_test(node)
    }
}
