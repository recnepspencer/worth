//! A recorded envelope stage may reach only producers document history can record.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;

impl<S: ApplicationSchema> CapturedAuthoredEditStage<S> {
    /// Refuse a stage whose outputs can reach a producer with workflow control
    /// or an external effect. The denial names that producer. Call this after
    /// the stage's output dependencies are captured and before any stage runs.
    pub fn require_history_recordable_producers(
        &self,
        application: &dyn crate::ports::ApplicationRuntimeRef<S>,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        application
            .runtime_inner()
            .installed_producers
            .history_refusal_among(&self.producers)
            .map_or(Ok(()), Err)
    }
}
