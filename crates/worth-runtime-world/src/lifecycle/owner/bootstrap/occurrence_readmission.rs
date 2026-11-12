//! Materialize an original occurrence with only its actual acting owners.
use super::*;
use crate::durability::{AdmittedWorldOccurrence, WorldReadmissionDenial as Denial};
use crate::history::{ReadmittedPublicationRecord, RecoveredWorldOwnerEvidence};

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
        use crate::durability::WorldOpenComponentIntent as Intent;
        use crate::history::RecoveredWorldOwnerEvidenceRef as Owners;
        let actual_intent = match owners.borrowed() {
            Owners::RelationalOnly(_) => Intent::RelationalOnly,
            Owners::SignalOnly(_) => Intent::SignalOnly,
            Owners::RelationalAndSignal { .. } => Intent::RelationalAndSignal,
        };
        let owners_match_open = actual_intent == open.component_intent;
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
}
