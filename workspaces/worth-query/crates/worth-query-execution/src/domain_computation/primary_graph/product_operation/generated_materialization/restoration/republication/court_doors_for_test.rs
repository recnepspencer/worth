//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn record_restored_generated_output_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        producer: ProducerQualification,
    ) {
        Self::record_restored_generated_output(
            self,
            execution,
            observation,
            correspondence,
            producer,
        )
    }
}
