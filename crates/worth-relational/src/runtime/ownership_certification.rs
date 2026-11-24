//! Bind the existing certification entries to the actual Runtime receiver.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::EntityId;
use crate::publication::ownership::{certification, CurrentOwnershipContactsForTest};
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{
    outcomes::transaction_commit_error::TransactionCommitError, primitives::WorkerIntentBatch,
};
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;

impl RelationalRuntime {
    /// Remove both current fact families from the real selected read image. The
    /// independently retained warm position image must still refuse the loss.
    #[doc(hidden)]
    pub fn probe_coherent_ownership_loss_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<(), TransactionCommitError> {
        certification::probe_coherent_ownership_loss(execution, basis, owner)
    }

    /// A warm read image with independent presence but no publication row.
    #[doc(hidden)]
    pub fn probe_unmatched_ownership_presence_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<(), TransactionCommitError> {
        certification::probe_unmatched_ownership_presence(execution, basis, owner)
    }

    /// Inspect the actual current aggregate memberships of this Query publication.
    /// Returned identifiers grant no mutation or publication standing.
    #[doc(hidden)]
    pub fn current_owned_aggregate_memberships_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<Vec<crate::identity::data::RelationId>, TransactionCommitError> {
        certification::current_owned_aggregate_memberships(execution, basis, owner)
    }

    /// Descriptive publication sources reached for one real Query output record.
    #[doc(hidden)]
    pub fn current_ownership_sources_for_record_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        record: EntityId,
    ) -> Result<Vec<EntityId>, TransactionCommitError> {
        certification::current_ownership_sources(execution, basis, record)
    }

    #[doc(hidden)]
    pub fn current_ownership_contacts_for_record_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        record: EntityId,
    ) -> Result<CurrentOwnershipContactsForTest, TransactionCommitError> {
        certification::current_ownership_contacts(execution, basis, record)
    }

    /// The actual cascade owner, distinct from ordinary one-hop custody reach.
    #[doc(hidden)]
    pub fn current_ownership_descendants_for_record_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        source: EntityId,
    ) -> Result<Vec<EntityId>, TransactionCommitError> {
        certification::current_ownership_descendants(execution, basis, source)
    }

    /// Read-only observation of actual one-hop reads during public Query edits.
    #[doc(hidden)]
    pub fn record_current_ownership_contacts_for_test<
        F: Fn(CurrentOwnershipContactsForTest) + 'static,
    >(
        &self,
        observer: F,
    ) -> impl Sized + use<F> {
        certification::record_current_ownership_contacts(observer)
    }

    /// Select an actual owned business edge; callers supply no publication,
    /// relation roster or ownership proof. Preparation never publishes an effect.
    #[doc(hidden)]
    pub fn prepare_owned_edge_retarget_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
        target: EntityId,
    ) -> Result<(), TransactionCommitError> {
        let intent = certification::owned_edge_retarget_intent(execution, basis, owner, target)?;
        let mut transaction = self
            .begin_branch_transaction(basis, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|cause| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    format!("actual endpoint probe admission: {cause:?}"),
                )
            })?;
        transaction
            .push_batch(
                WorkerIntentBatch::new("external-owned-edge-retarget").push(intent),
                execution.allocation_policy(),
            )
            .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        let _unpublished = self.prepare_branch_transaction(execution, transaction)?;
        Ok(())
    }

    /// Exercise both existing cold route doors using a real Query publication's
    /// canonical route. Only descriptive identifiers enter this readonly probe.
    #[doc(hidden)]
    pub fn probe_current_history_contradiction_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        publication: crate::history::data::CommitId,
        source: EntityId,
    ) -> Result<Vec<Result<(), TransactionCommitError>>, TransactionCommitError> {
        let envelope = self
            .history
            .canonical_envelope(publication)
            .ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "real Query publication history unavailable",
                )
            })?;
        let meaning = certification::source_route_meaning(&envelope, source)?;
        let absence = self
            .readmit_required_output_route_absence(execution, installed, basis, &meaning)
            .map(|_| ());
        let retirement = self
            .readmit_native_output_transition_successor(execution, installed, basis, &meaning)
            .map(|_| ());
        Ok(vec![absence, retirement])
    }
}
