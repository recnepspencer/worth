//! Descriptive history selection comes from the actual retained observation.
use super::ManagedIndexSourceObservation;
use crate::history::access::relevant_writes::managed_source::selected_catalog::ManagedSourceHistoryReads;
use crate::history::access::relevant_writes::RelationalAuthoredManagedSourceSelector;
use crate::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use worth_execution::ExecutionRequest;
impl ManagedIndexSourceObservation {
    /// Borrow this scope's actual canonical impact and historical pure material.
    /// This grants no historical READ or performed-correction permission.
    pub fn authored_history_impact<'a>(
        &self,
        reads: impl Into<ManagedSourceHistoryReads<'a>>,
        execution: ExecutionRequest<'_, '_>,
        occurrence: &crate::history::access::relevant_writes::RelationalAuthoredWriteOccurrence,
    ) -> Result<
        crate::history::access::relevant_writes::RelationalAuthoredManagedSourceImpact,
        Denial,
    > {
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
        crate::history::access::relevant_writes::run(execution, |context| {
            crate::history::access::relevant_writes::admit_work(context, width)?;
            if occurrence.receipt().branch_id != *self.selected.branch_id() {
                return Err(Denial::ForeignOwner);
            }
            Ok(())
        })?;
        let reads = reads.into();
        let selector = self.authored_history_selector_from(&reads, execution)?;
        crate::history::access::relevant_writes::source_impact(
            reads.runtime_instance_id,
            execution,
            &self.affinity,
            selector,
            occurrence,
        )
    }
    /// Select metadata for this actual program/scope, not a reconstructed key.
    /// Retained observation custody grants no Source READ or correction permission.
    pub fn authored_history_selector<'a>(
        &self,
        reads: impl Into<ManagedSourceHistoryReads<'a>>,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        self.authored_history_selector_from(&reads.into(), execution)
    }

    fn authored_history_selector_from(
        &self,
        reads: &ManagedSourceHistoryReads<'_>,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        execution.consult().map_err(Denial::Request)?;
        if self.selected.runtime_instance_id() != reads.runtime_instance_id {
            return Err(Denial::ForeignOwner);
        }
        reads.select(execution, &self.record.program, self.record.scope)
    }
}

impl super::ManagedSourceRootImage {
    pub(crate) fn history_owner(
        &self,
    ) -> Option<std::sync::Arc<super::affinity::ManagedSourceCellAffinity>> {
        self.image
            .payload
            .is::<super::ManagedIndexBranchImage>()
            .then(|| std::sync::Arc::clone(&self.affinity))
    }
    /// The directory receiver creates only funded weak metadata, not a strong
    /// temporary that could be destroyed after concurrent cache eviction.
    pub(crate) fn history_payload_receipt(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &super::ManagedIndexSourceProgram,
        scope: crate::identity::data::EntityId,
        budget: &crate::validation::custom_rule::work_meter::CustomPreparationBudget,
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
