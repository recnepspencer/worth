//! Materialize an original occurrence with only its actual acting owners.
use super::*;
use crate::durability::{AdmittedWorldOccurrence, WorldReadmissionDenial as Denial};
use crate::history::{ReadmittedPublicationRecord, RecoveredWorldOwnerEvidence};
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::RelationalRecoveredPublicationCustody;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn prepare_readmitted_occurrence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &ProductBranchObservation,
        occurrence: &AdmittedWorldOccurrence,
        owners: RecoveredWorldOwnerEvidence,
        bridge: worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Result<std::sync::Arc<ReadmittedPublicationRecord>, (RecoveredWorldOwnerEvidence, Denial)>
    {
        let location = occurrence.location();
        let open = occurrence.open();
        let owners_match_open = match (open.component_intent, owners.borrowed()) {
            (
                crate::durability::WorldOpenComponentIntent::RelationalOnly,
                crate::history::RecoveredWorldOwnerEvidenceRef::RelationalOnly(_),
            ) => true,
            (
                crate::durability::WorldOpenComponentIntent::RelationalOnly,
                crate::history::RecoveredWorldOwnerEvidenceRef::SignalOnly(_)
                | crate::history::RecoveredWorldOwnerEvidenceRef::RelationalAndSignal { .. },
            ) => false,
        };
        if !owners_match_open
            || Some(location.owner()) != self.owner_identity().durable()
            || location.attempt() != open.association.attempt()
            || location.commit() != open.commit
        {
            return Err((owners, Denial::OccurrenceEvidenceMismatch));
        }
        match self.prepare_readmitted_successor(execution, before, open, owners.borrowed(), bridge)
        {
            Err(cause) => Err((owners, cause)),
            Ok(prepared) => Ok(ReadmittedPublicationRecord::new(
                prepared.record,
                before.snapshot().clone(),
                prepared.after,
                prepared.attempt,
                location.clone(),
                occurrence.publishing_incarnation(),
                owners,
            )),
        }
    }

    pub(crate) fn install_readmitted_occurrence(
        &self,
        record: &std::sync::Arc<ReadmittedPublicationRecord>,
        capacities: &mut Vec<crate::history::ReservedCompositeCommitCapacity>,
    ) -> Result<ProductBranchObservation, Denial> {
        let commit = record.after.shared_commit();
        let basis = commit.basis();
        let capacity = self
            .state
            .history
            .reserve_readmitted_capacity(commit.identity().clone(), commit.parent().clone())
            .map_err(Denial::HistoryAdmission)?;
        capacities.push(capacity);
        let head = self
            .state
            .retention
            .issue_product_head(basis)
            .map_err(|cause| Denial::HistoryRetention(map_retention_denial(cause)))?;
        let pins = head
            .fork_history(basis)
            .map_err(|_| Denial::RecoveryCapacity)?;
        capacities
            .last_mut()
            .expect("same retained capacity")
            .install_readmitted(commit.clone(), pins, record.clone())
            .map_err(Denial::HistoryAdmission)?;
        let history = self
            .state
            .history
            .protect_product_head(&commit)
            .map_err(Denial::HistoryAdmission)?;
        let protection =
            ProductBranchHeadProtection::bootstrap_issued(record.after.clone(), head, history)
                .map_err(|_| Denial::BranchLineage)?;
        let cell = self
            .state
            .branches
            .branch_cell(record.after.branch_identity())
            .ok_or(Denial::BranchLineage)?;
        cell.restore_readmitted(record, protection)
            .map_err(|_| Denial::BranchLineage)?;
        cell.observe(&self.state.history, &self.state.retention)
            .map_err(|_| Denial::RecoveryCapacity)
    }

    pub(crate) fn prepare_original_recovered_image(
        &self,
        before: &ProductBranchObservation,
        admitted_open: &crate::durability::AdmittedWorldOpenAttempt,
        native: &RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
        identity: worth_relational::facade::history::RelationalCommitIdentity,
    ) -> Result<
        (
            ProductBranchReferenceSnapshot,
            crate::identity::CompositePublicationAttemptIdentity,
        ),
        Denial,
    > {
        let open = admitted_open.open();
        let owner = self.owner_identity();
        (|| {
            if native.custody().predecessor().admission_identity()
                != before.basis().relational_basis().admission_identity()
            {
                return Err(Denial::OccurrenceEvidenceMismatch);
            }
            let commit = crate::identity::readmission::readmit_commit(owner, open.commit)
                .ok_or(Denial::ForeignOwner)?;
            let attempt =
                crate::identity::readmission::readmit_attempt(owner, open.association.attempt())
                    .ok_or(Denial::ForeignOwner)?;
            let basis = crate::basis::admit_validated(
                &self
                    .state
                    .identities
                    .lock()
                    .unwrap_or_else(|e| e.into_inner()),
                native.custody().successor().clone(),
                before.basis().signal_basis().clone(),
                before.basis().correspondence_basis().clone(),
            );
            let results = crate::publication::CompositeOwnerExecutionResults::retained()
                .with_relational_settled_receipt(
                    identity,
                    basis.relational_basis().clone(),
                    native.custody().effect().subject().receipt().clone(),
                );
            let commit = CompositeRuntimeWorldCommit::from_ordinary_publication(
                commit,
                before.snapshot().commit(),
                basis,
                attempt.clone(),
                &results,
                None,
            )
            .map_err(|_| Denial::OccurrenceEvidenceMismatch)?;
            let generation = before
                .reference_generation()
                .advance()
                .map_err(Denial::IdentityExhaustion)?;
            let after = ProductBranchReferenceSnapshot::owner_issued(
                owner,
                before.branch_identity().clone(),
                before.lifecycle_incarnation(),
                generation,
                std::sync::Arc::new(commit),
            )
            .map_err(|_| Denial::BranchLineage)?;
            Ok((after, attempt))
        })()
    }
}
