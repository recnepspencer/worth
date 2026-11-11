//! The same Home's recorded dispatch custody is admitted before service.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryDispatchJournalDenial as Denial, QueryDispatchMedium,
};

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
            .runtime
            .primary_provider
            .readmit_dispatch_outboxes(
                execution,
                &application.runtime.product_runtime.owner.observation_port(),
                &application.runtime.product_runtime.owner.inspection_port(),
            )
            .map_err(Denial::DispatchOutbox)?
        {
            application
                .readmit_inbound_terminal_sources(execution, unpublished_native)
                .map_err(Denial::Inbound)?;
            application.runtime.product_runtime.admit_directory_capability(
                crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability::InboundCompletion,
            ).map_err(Denial::DispatchCapability)?;
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
        let owner = application.runtime.product_runtime.owner.owner_identity();
        if owner.durable().map(|owner| owner.home()) != Some(dispatch.home()) {
            return Err(Denial::ForeignHome);
        }
        let fence = owner.home_incarnation_fence().ok_or(Denial::ForeignHome)?;
        if fence.admit(dispatch.incarnation().fence()).is_err() {
            return Err(Denial::StaleIncarnation);
        }
        let admission = application.runtime.dispatch_journal.readmit(
            dispatch,
            execution,
            application
                .runtime
                .primary_provider
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
