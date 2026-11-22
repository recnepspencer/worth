//! The same Home's recorded dispatch custody is admitted before service.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryDispatchJournalDenial as Denial, QueryDispatchMedium,
};
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub(super) fn admit_dispatch_contracts(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        unpublished_native: &std::collections::BTreeSet<
            worth_relational::facade::history::CommitId,
        >,
    ) -> Result<(), QueryHomeCustodyReadmissionDenial> {
        use QueryHomeCustodyReadmissionDenial as Denial;
        let application = self.runtime.runtime();
        if application
            .inner
            .runtime
            .primary_provider_ref()
            .readmit_dispatch_outboxes(
                execution,
                &application
                    .inner
                    .runtime
                    .product_runtime_ref()
                    .owner_ref()
                    .observation_port(),
                &application
                    .inner
                    .runtime
                    .product_runtime_ref()
                    .owner_ref()
                    .inspection_port(),
            )
            .map_err(Denial::DispatchOutbox)?
        {
            application
                .readmit_inbound_terminal_sources(execution, unpublished_native)
                .map_err(Denial::Inbound)?;
        }
        Ok(())
    }

    /// Folds recorded DATA only. No attempt is contacted, replayed or reissued,
    /// and a retried admission must reach the identical protected prefix.
    pub(super) fn admit_dispatch_custody(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        dispatch: std::sync::Arc<dyn QueryDispatchMedium>,
    ) -> Result<(), Denial> {
        let application = self.runtime.runtime();
        let owner = application
            .inner
            .runtime
            .product_runtime_ref()
            .owner_ref()
            .owner_identity();
        if owner.durable().map(|owner| owner.home()) != Some(dispatch.home()) {
            return Err(Denial::ForeignHome);
        }
        let fence = owner.home_incarnation_fence().ok_or(Denial::ForeignHome)?;
        if fence.admit(dispatch.incarnation().fence()).is_err() {
            return Err(Denial::StaleIncarnation);
        }
        let admission = application.inner.runtime.dispatch_journal_ref().readmit(
            dispatch,
            execution,
            application
                .inner
                .runtime
                .primary_provider_ref()
                .dispatch_readmission_catalog(),
        )?;
        match self.dispatch_custody_admission.get() {
            Some(existing) => {
                if existing.position() != admission.position()
                    || existing.digest() != admission.digest()
                    || existing.payload_digest() != admission.payload_digest()
                {
                    return Err(Denial::PrefixMismatch);
                }
                Ok(())
            }
            None => self
                .dispatch_custody_admission
                .set(admission)
                .map_err(|_| Denial::OwnerUnavailable),
        }
    }
}
