//! Materialize an original occurrence using concrete recovered Native custody.
use super::*;
use crate::durability::{AdmittedWorldOccurrence, WorldReadmissionDenial as Denial};
use crate::history::ReadmittedPublicationRecord;
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
        before: &ProductBranchObservation,
        occurrence: &AdmittedWorldOccurrence,
        native: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
        identity: worth_relational::facade::history::RelationalCommitIdentity,
    ) -> Result<
        std::sync::Arc<ReadmittedPublicationRecord>,
        (
            RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
            Denial,
        ),
    > {
        if Some(occurrence.location().owner()) != self.owner_identity().durable()
            || occurrence.location().attempt() != occurrence.open().association.attempt()
            || occurrence.location().commit() != occurrence.open().commit
        {
            return Err((native, Denial::OccurrenceEvidenceMismatch));
        }
        let record = self.prepare_original_recovered_publication(
            before,
            occurrence.admitted_open(),
            occurrence.publishing_incarnation(),
            native,
            identity,
        )?;
        Ok(record.seal(occurrence))
    }

    pub(crate) fn prepare_original_recovered_publication(
        &self,
        before: &ProductBranchObservation,
        admitted_open: &crate::durability::AdmittedWorldOpenAttempt,
        incarnation: worth_foundational::facade::HomeIncarnation,
        native: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
        identity: worth_relational::facade::history::RelationalCommitIdentity,
    ) -> Result<
        std::sync::Arc<crate::history::PreparedReadmittedPublicationRecord>,
        (
            RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
            Denial,
        ),
    > {
        let prepared =
            self.prepare_original_recovered_image(before, admitted_open, &native, identity);
        match prepared {
            Err(cause) => Err((native, cause)),
            Ok((after, attempt)) => Ok(ReadmittedPublicationRecord::prepare_original(
                before.snapshot().clone(),
                after,
                attempt,
                incarnation,
                native,
            )),
        }
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
            .reserve_commit_capacity(commit.identity().clone(), commit.parent().clone())
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
}
