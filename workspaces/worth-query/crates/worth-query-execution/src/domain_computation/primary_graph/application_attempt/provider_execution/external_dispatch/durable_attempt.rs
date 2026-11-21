pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::durable_attempt::*;

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial;
use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

use super::denial::WorthQueryExternalDispatchPreparationDenial;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Only a memory-only owner dispatches without a durable record. A durable
    /// owner whose journal is not attached refuses before any contact.
    pub(super) fn begin_durable_dispatch_contact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: &WorthQueryDispatchOutboxRecord,
    ) -> Result<DurableDispatchContact<'_>, WorthQueryExternalDispatchPreparationDenial> {
        let journal = &self.runtime.dispatch_journal;
        let attempt = if journal.is_durable() {
            Some(
                journal
                    .record_attempt(*record.correlation().digest(), execution)
                    .map_err(attempt_record_denial)?,
            )
        } else if self
            .runtime
            .product_runtime
            .owner
            .owner_identity()
            .durable()
            .is_some()
        {
            return Err(attempt_record_denial(
                QueryDispatchJournalDenial::JournalUnavailable,
            ));
        } else {
            None
        };
        Ok(DurableDispatchContact::new(
            journal,
            attempt,
            *record.correlation().digest(),
        ))
    }
}
