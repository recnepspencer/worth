//! Descriptive history selection comes from the actual retained observation.
use super::ManagedIndexSourceObservation;
use crate::history::{
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial as Denial,
};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;
impl ManagedIndexSourceObservation {
    /// Borrow this scope's actual canonical impact and historical pure material.
    /// This grants no historical READ or performed-correction permission.
    pub fn authored_history_impact(
        &self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'_, '_>,
        occurrence: &crate::history::RelationalAuthoredWriteOccurrence,
    ) -> Result<crate::history::RelationalAuthoredManagedSourceImpact, Denial> {
        execution.consult().map_err(Denial::Request)?;
        let width = occurrence
            .receipt()
            .branch_id
            .0
            .len()
            .checked_add(self.selected.branch_id().0.len())
            .and_then(|bytes| bytes.checked_add(1))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::SizeOverflow)?;
        crate::history::run_authored_write_stage(execution, |context| {
            crate::history::admit_authored_write_work(context, width)?;
            if occurrence.receipt().branch_id != *self.selected.branch_id() {
                return Err(Denial::ForeignOwner);
            }
            Ok(())
        })?;
        crate::history::source_impact(runtime, execution, self, occurrence)
    }
    /// Select metadata for this actual program/scope, not a reconstructed key.
    /// Retained observation custody grants no Source READ or correction permission.
    pub fn authored_history_selector(
        &self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        execution.consult().map_err(Denial::Request)?;
        if self.selected.runtime_instance_id() != runtime.runtime_instance_id() {
            return Err(Denial::ForeignOwner);
        }
        runtime.authored_managed_source_selector(execution, &self.record.program, self.record.scope)
    }
}

/// Actual cell identity only: no physical basis, image, interpretation or READ.
pub(crate) struct ManagedSourceHistoryOwner(
    std::sync::Arc<super::affinity::ManagedSourceCellAffinity>,
);
impl ManagedSourceHistoryOwner {
    pub(crate) fn admits(&self, observation: &ManagedIndexSourceObservation) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &observation.affinity)
    }
}
impl super::ManagedSourceRootImage {
    pub(crate) fn history_owner(&self) -> Option<ManagedSourceHistoryOwner> {
        self.image
            .payload
            .is::<super::ManagedIndexBranchImage>()
            .then(|| ManagedSourceHistoryOwner(std::sync::Arc::clone(&self.affinity)))
    }
    /// The directory receiver creates only funded weak metadata, not a strong
    /// temporary that could be destroyed after concurrent cache eviction.
    pub(crate) fn history_payload_receipt(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &super::ManagedIndexSourceProgram,
        scope: crate::identity::data::EntityId,
        budget: &crate::validation::CustomPreparationBudget,
    ) -> Result<
        Option<crate::validation::candidate_source::opaque_custody::FundedSourceMaterialReceipt>,
        super::Denial,
    > {
        let image = self
            .image
            .payload
            .downcast_ref::<super::ManagedIndexBranchImage>()
            .ok_or(super::Denial::ForeignSource)?;
        image.retained_history_receipt(execution, program, scope, budget)
    }
}
