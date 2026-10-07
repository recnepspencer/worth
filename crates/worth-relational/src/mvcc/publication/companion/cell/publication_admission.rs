//! A speculative preflight cell has no readable lookup projection until Native cutover.
use super::{
    CompanionBranchCell, CompanionBranchCellCore, CompanionPreflightStop,
    PublicationCompanionPreflight, ReservedCompanionBranchCell,
};
use std::sync::{Arc, OnceLock};

pub(in crate::mvcc::publication::companion) struct PublicationCellAdmission {
    cell: OnceLock<Arc<CompanionBranchCellCore>>,
}
impl PublicationCellAdmission {
    pub(in crate::mvcc::publication::companion) fn installed(
        &self,
        cell: &Arc<CompanionBranchCellCore>,
    ) {
        // The exact non-cloneable effect reaches this only after its head comparison.
        // OnceLock publication performs no allocation or caller callback.
        let _ = self.cell.set(Arc::clone(cell));
    }
}

/// Preflight custody bound to its creator. It cannot read or edit a published image.
pub struct PreparedCompanionBranchCell<T: Send + Sync + 'static> {
    cell: CompanionBranchCell<T>,
    admission: Arc<PublicationCellAdmission>,
    candidate_id: u64,
}
impl<T: Send + Sync + 'static> PreparedCompanionBranchCell<T> {
    pub(in crate::mvcc::publication::companion) fn new_selected(
        binding: &super::super::preflight::CandidateCompanionBinding,
        initial: Arc<T>,
        retention: super::super::registration::CompanionCellRetention,
    ) -> Self {
        Self {
            cell: CompanionBranchCell::new_selected(binding, initial, retention),
            admission: Arc::new(PublicationCellAdmission {
                cell: OnceLock::new(),
            }),
            candidate_id: binding.candidate_id,
        }
    }
    pub fn reserve_preflight(
        &self,
        context: &PublicationCompanionPreflight<'_>,
    ) -> Result<ReservedCompanionBranchCell<T>, CompanionPreflightStop> {
        if context.candidate_id() != self.candidate_id {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        let mut reserved = self.cell.reserve_preflight(context)?;
        reserved.publication_admission = Some(Arc::clone(&self.admission));
        Ok(reserved)
    }
    /// Retaining this slot retains no candidate cell. Abort leaves it empty.
    pub fn lookup_slot(&self) -> CompanionBranchCellSlot<T> {
        CompanionBranchCellSlot {
            source: SlotSource::Cutover(Arc::clone(&self.admission)),
        }
    }
}

enum SlotSource<T: Send + Sync + 'static> {
    Head(CompanionBranchCell<T>),
    Cutover(Arc<PublicationCellAdmission>),
}
/// A Query lookup can obtain a cell only from Native head admission or successful cutover.
pub struct CompanionBranchCellSlot<T: Send + Sync + 'static> {
    source: SlotSource<T>,
}
impl<T: Send + Sync + 'static> Clone for CompanionBranchCellSlot<T> {
    fn clone(&self) -> Self {
        Self {
            source: match &self.source {
                SlotSource::Head(cell) => SlotSource::Head(cell.clone()),
                SlotSource::Cutover(admission) => SlotSource::Cutover(Arc::clone(admission)),
            },
        }
    }
}
impl<T: Send + Sync + 'static> CompanionBranchCellSlot<T> {
    pub fn admitted(&self) -> Option<CompanionBranchCell<T>> {
        match &self.source {
            SlotSource::Head(cell) => Some(cell.clone()),
            SlotSource::Cutover(admission) => {
                admission.cell.get().map(|core| CompanionBranchCell {
                    core: Arc::clone(core),
                    _payload: std::marker::PhantomData,
                })
            }
        }
    }
    /// Additional deferred-slot custody beyond its inline map value; the cell
    /// has separate Native retention. Even an aborted slot owns this allocation.
    pub const fn maximum_deferred_retained_bytes() -> u64 {
        super::super::preflight::arc_allocation_bound::<PublicationCellAdmission>()
    }
}
impl<T: Send + Sync + 'static> CompanionBranchCell<T> {
    pub fn into_lookup_slot(self) -> CompanionBranchCellSlot<T> {
        CompanionBranchCellSlot {
            source: SlotSource::Head(self),
        }
    }
}
