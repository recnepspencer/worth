//! Consume the publication body phase at the bound transaction.
use crate::mvcc::{
    RelationalPublicationAdoptionRequest, RelationalPublicationAppendix,
    RelationalPublicationBodySelection,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;

impl crate::mvcc::BranchBoundRelationalTransaction {
    /// Consume Native's exact body/appendix phase. An omitted appendix can only
    /// be adopted; it can never accidentally become an ordinary publication.
    pub fn push_publication_body(
        &mut self,
        selection: RelationalPublicationBodySelection,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let (batch, adoption, basis) = selection.into_parts();
        if self.basis != basis || self.appendix_adoption.is_some() {
            return Err(
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "publication body belongs to another transaction basis",
                ),
            );
        }
        self.push_batch(batch, policy)
            .map_err(|stop| TransactionCommitError::conflict(stop.into_conflict()))?;
        self.appendix_adoption = adoption;
        Ok(())
    }
}

impl super::RelationalRuntime {
    /// The judgment allocates no record or symbol identities. A real publication
    /// stages the original receipt in its original place and follows the normal
    /// canonical plan, retaining that plan's allocation and byte ordering.
    pub fn prepare_publication_appendix(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
        body: crate::transactions::data::primitives::WorkerIntentBatch,
        content: &crate::content::RelationalContentAttachments,
        appendix: RelationalPublicationAppendix,
        adoption: Option<RelationalPublicationAdoptionRequest<'_>>,
    ) -> Result<RelationalPublicationBodySelection, TransactionCommitError> {
        crate::mvcc::publication::application_publication::prepare_publication_appendix(
            self.runtime_instance_id(),
            &self.services.symbols,
            execution,
            basis,
            body,
            content,
            appendix,
            adoption,
        )
    }
}
