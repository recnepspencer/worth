pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::durable_attempt::*;

use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial;
use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

use super::denial::WorthQueryExternalDispatchPreparationDenial;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Only a memory-only owner dispatches without a durable record. A durable
    /// owner whose journal is not attached refuses before any contact.
    pub fn begin_durable_dispatch_contact(
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

    /// The latest custody this Home's dispatch journal durably records for
    /// the effect with this correlation token (the token its outbox row and
    /// every transport request carry). `None` for a memory-only runtime and
    /// for an effect no attempt has been recorded for. Recorded DATA: it
    /// grants no dispatch authority.
    pub fn durable_dispatch_custody(
        &self,
        correlation_token: &[u8; 32],
    ) -> Option<
        crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCustody,
    > {
        self.runtime
            .dispatch_journal
            .custody(&worth_foundational::facade::CanonicalDigestId::new(
                *correlation_token,
            ))
    }
}
