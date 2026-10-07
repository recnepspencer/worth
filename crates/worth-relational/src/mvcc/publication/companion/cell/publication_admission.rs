//! A speculative preflight cell has no readable lookup projection until Native cutover.
use super::{
    CompanionBranchCell, CompanionBranchCellCore, CompanionPreflightStop,
    PublicationCompanionPreflight, ReservedCompanionBranchCell,
};
use std::sync::{Arc, OnceLock, Weak};

pub(in crate::mvcc::publication::companion) struct PublicationCellAdmission {
    cell: OnceLock<Arc<CompanionBranchCellCore>>,
    pending: Weak<CompanionBranchCellCore>,
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
        let cell = CompanionBranchCell::new_selected(binding, initial, retention);
        let pending = Arc::downgrade(&cell.core);
        Self {
            cell,
            admission: Arc::new(PublicationCellAdmission {
                cell: OnceLock::new(),
                pending,
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
    /// Adopt a live, unpublished first candidate's exact cell. Native validates
    /// the runtime, branch, registration and expected head before rebinding the
    /// prepared wrapper to this candidate. Its topology pin admits one writer;
    /// a second in-flight effect receives TopologyPending without replacing the
    /// shared lookup. Once every candidate drops, the weak pending core expires.
    pub fn pending_for_preflight(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<Option<PreparedCompanionBranchCell<T>>, CompanionPreflightStop> {
        let work = u64::try_from(context.branch_id().0.len())
            .ok()
            .and_then(|bytes| bytes.checked_add(1))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        context.claim_work(work)?;
        let SlotSource::Cutover(admission) = &self.source else {
            return Ok(None);
        };
        if admission.cell.get().is_some() {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let Some(core) = admission.pending.upgrade() else {
            return Ok(None);
        };
        context.validate_cell(&core)?;
        let cell = CompanionBranchCell {
            core,
            _payload: std::marker::PhantomData,
        };
        let image = cell.read_image();
        if image.root_id() != context.expected_root_id()
            || image.commit_id() != context.expected_commit_id()
        {
            // The first cutover can update its image just before admitting
            // the lookup. A live slot is never replaced during that window.
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        Ok(Some(PreparedCompanionBranchCell {
            cell,
            admission: Arc::clone(admission),
            candidate_id: context.candidate_id(),
        }))
    }

    /// Deferred-slot storage beyond its inline map value, including the core
    /// allocation retained by its weak link after abort. Live Native cell custody
    /// remains separate, so its bound can conservatively overlap this storage.
    pub const fn maximum_deferred_retained_bytes() -> u64 {
        super::super::preflight::arc_allocation_bound::<PublicationCellAdmission>()
            // After abort the Weak can keep the deinitialized core allocation
            // alive beyond its Native ticket. The deferred-slot ticket prepays
            // that storage too; live Native custody can conservatively overlap.
            .saturating_add(super::super::preflight::arc_allocation_bound::<
                CompanionBranchCellCore,
            >())
    }
}
impl<T: Send + Sync + 'static> CompanionBranchCell<T> {
    pub fn into_lookup_slot(self) -> CompanionBranchCellSlot<T> {
        CompanionBranchCellSlot {
            source: SlotSource::Head(self),
        }
    }
}
