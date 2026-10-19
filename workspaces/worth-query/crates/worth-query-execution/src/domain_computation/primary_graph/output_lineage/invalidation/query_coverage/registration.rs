//! Exact native read registration, full-verification re-establishment and release.
use super::super::{
    admission::RetainedIndexAdmission,
    mark_state::MarkState,
    settlement::{self, PostingBindings},
    source_alignment::BranchMarkRoot,
};
use super::*;
use worth_relational::facade::mvcc::{CompanionBranchCell, CompanionBranchImage};

pub(super) fn is_live(
    image: &CompanionBranchImage<BranchMarkRoot>,
    selected: &PositionedRelationalSnapshot,
) -> bool {
    image.root_id() == selected.root_id()
        && image.commit_id() == selected.commit_id()
        && image.position() == selected.position()
}
pub(super) fn same_source(
    owner: &SourceInvalidationOwner,
    coverage: &QueryFactCoverage,
    selected: &PositionedRelationalSnapshot,
) -> bool {
    selected.runtime_instance_id() == owner.runtime_instance_id
        && coverage.basis.runtime_instance_id() == owner.runtime_instance_id
        && selected.branch_id() == coverage.basis.branch_id()
        && selected.position() >= coverage.basis.position()
}
impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn reestablish_query_read(
        &self,
        coverage: &QueryFactCoverage,
        selected: &PositionedRelationalSnapshot,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), QueryCoverageStop> {
        policy.check_live()?;
        if !same_source(self, coverage, selected) {
            return Ok(());
        }
        let Some(cell) = self.cell_for_read(selected, admission)? else {
            return Ok(());
        };
        let image = cell.read_image();
        if !is_live(&image, selected) {
            return Ok(());
        } // Historical truth is read-only.
        self.install_query_read(coverage, selected, cell, image, policy, admission)
    }

    pub(super) fn install_query_read(
        &self,
        coverage: &QueryFactCoverage,
        selected: &PositionedRelationalSnapshot,
        cell: CompanionBranchCell<BranchMarkRoot>,
        image: CompanionBranchImage<BranchMarkRoot>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), QueryCoverageStop> {
        policy.check_live()?;
        // Serializes ONLY this registration locator, never source publication.
        let mut slot = coverage
            .slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = admission.index_checkpoint();
        admission.bytes(
            index_capacity::arc_bytes::<MarkState>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut next = (*image.payload().current).clone();
        admission.ordered_read(next.posting_bindings.len())?;
        let retained = slot.and_then(|id| next.posting_bindings.get(&id).map(|binding| (id, binding)))
            .filter(|(_, binding)| matches!(binding.as_ref(), PostingBindings::QueryRead(read) if read.belongs_to(coverage)));
        let id = match retained {
            Some((id, _)) => id,
            None => {
                let id = next.next_posting_projection;
                next.next_posting_projection = id
                    .checked_add(1)
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                settlement::install_physical(&mut next, id, &coverage.projection.keys, admission)?;
                id
            }
        };
        admission.index_bytes(
            index_capacity::arc_bytes::<PostingBindings>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        admission.index_edit::<u64, Arc<PostingBindings>>(next.posting_bindings.len())?;
        let binding = QueryReadBinding {
            projection: QueryReadBacking::Live(Arc::clone(&coverage.projection)),
            identity: Arc::downgrade(&coverage.identity),
            established: selected.position(),
            epoch: next.delivery_epoch,
            dirty: None,
            _identity_capacity: Arc::clone(&coverage._capacity),
        };
        next.posting_bindings
            .insert(id, Arc::new(PostingBindings::QueryRead(binding)));
        policy.check_live()?;
        self.install_live_state(cell, image, (next, before), admission)?;
        *slot = Some(id); // Only successful native CAS changes this locator.
        policy.check_live()?;
        Ok(())
    }

    pub(super) fn release_query_read(&self, coverage: &QueryFactCoverage) {
        let slot = *coverage
            .slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(slot) = slot else {
            return;
        };
        let mut admission = self.edit_admission();
        let mut release = || -> Result<(), super::super::SettlementVerificationStop> {
            let Some(cell) = self.cell_for_read(&coverage.basis, &mut admission)? else {
                return Ok(());
            };
            let image = cell.read_image();
            admission.ordered_read(image.payload().current.posting_bindings.len())?;
            let Some(PostingBindings::QueryRead(read)) = image
                .payload()
                .current
                .posting_bindings
                .get(&slot)
                .map(Arc::as_ref)
            else {
                return Ok(());
            };
            if !read.belongs_to(coverage) {
                return Ok(());
            }
            let before = admission.index_checkpoint();
            admission.bytes(
                index_capacity::arc_bytes::<MarkState>()
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
            )?;
            let mut next = (*image.payload().current).clone();
            settlement::remove_physical(
                &mut next,
                slot,
                &coverage.projection.keys,
                &mut admission,
            )?;
            self.install_live_state(cell, image, (next, before), &mut admission)
        };
        // Drop cannot certify a read or retry source effects. A racing/refused
        // edit leaves only a dead weak entry, reclaimed by its next matched
        // native bucket or derived eviction; its backing stays fully funded.
        let _ = release();
    }
}
