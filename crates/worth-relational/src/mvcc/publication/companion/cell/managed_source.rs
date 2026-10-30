//! Source observations retain selected data/affinity, never newer live cache custody.
use super::*;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::{
    ManagedIndexBranchImage, ManagedIndexSourceAssociation, ManagedIndexSourceAvailability,
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use std::sync::Weak;
pub(in crate::mvcc::publication::companion) mod affinity;
mod correspondence;

pub struct ManagedIndexSourceObservation {
    pub(in crate::mvcc) core: Weak<CompanionBranchCellCore>,
    affinity: Arc<affinity::ManagedSourceCellAffinity>,
    pub(in crate::mvcc) record: Arc<ManagedIndexSourceAssociation>,
    payload: Arc<dyn Any + Send + Sync>,
    selected: PositionedRelationalSnapshot,
    _metadata_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// One matching physical image follows the Native root's actual retained life.
/// The weak cell link cannot keep a newer branch cache alive through an old root.
#[derive(Clone)]
pub(crate) struct ManagedSourceRootImage {
    pub(in crate::mvcc) core: Weak<CompanionBranchCellCore>,
    pub(in crate::mvcc::publication::companion) affinity: Arc<affinity::ManagedSourceCellAffinity>,
    pub(in crate::mvcc) image: Arc<CompanionRootImage>,
    pub(in crate::mvcc) _fork_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl std::fmt::Debug for ManagedSourceRootImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSourceRootImage")
            .field("root", &self.image.root_id)
            .finish_non_exhaustive()
    }
}
impl ManagedSourceRootImage {
    /// Cleanup uses the genuine selected image and current registration; it
    /// cannot manufacture an association or borrow Source content.
    pub(crate) fn evict_selected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: &PositionedRelationalSnapshot,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        registration_generation: u64,
    ) -> Result<
        Option<crate::validation::candidate_source::ManagedIndexSourceCacheRetirement>,
        Denial,
    > {
        self.admit_selected(selected, registration_generation)?;
        let image = self
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        let retired = image
            .evict_source(execution, program, scope)
            .map_err(Denial::Publication)?;
        Ok(retired)
    }

    pub(crate) fn observe(
        &self,
        selected: PositionedRelationalSnapshot,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        registration_generation: u64,
    ) -> Result<ManagedIndexSourceObservation, Denial> {
        self.admit_selected(&selected, registration_generation)?;
        let image = self
            .image
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(Denial::ForeignSource)?;
        let record = image
            .retained(program, scope)?
            .ok_or(Denial::RootUnavailable)?;
        if record.physical_availability() != ManagedIndexSourceAvailability::Available {
            return Err(Denial::RootUnavailable);
        }
        let payload = record.payload().cloned().ok_or(Denial::RootUnavailable)?;
        Ok(ManagedIndexSourceObservation {
            core: self.core.clone(),
            affinity: Arc::clone(&self.affinity),
            record: Arc::clone(&record.association),
            payload,
            selected,
            _metadata_memory: memory,
        })
    }
}
impl ManagedIndexSourceObservation {
    pub fn payload(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.payload
    }
    /// Metadata affinity only. The receiving Query session still admits READ.
    pub fn matches_snapshot(&self, selected: &PositionedRelationalSnapshot) -> bool {
        &self.selected == selected
    }
    /// The Native publication that last replaced this complete Source support.
    /// Diagnostic identity alone cannot attach backing to a physical view.
    pub fn source_revision(&self) -> CommitId {
        self.record.revision
    }
    pub fn same_source(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.affinity, &other.affinity) && Arc::ptr_eq(&self.record, &other.record)
    }

    /// Compare the complete retained Source support at an actual Native image.
    /// This is currentness only; it does not admit a principal or Source READ.
    pub(crate) fn remains_current_at(
        &self,
        selected: &PositionedRelationalSnapshot,
        registration_generation: u64,
    ) -> bool {
        if self
            .affinity
            .admit(selected, registration_generation)
            .is_err()
        {
            return false;
        }
        let Some(core) = self.core.upgrade() else {
            return false;
        };
        let current = core
            .current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        current.root_id == selected.root_id()
            && current.commit_id == selected.commit_id()
            && current.position() == selected.position()
            && current
                .payload
                .downcast_ref::<ManagedIndexBranchImage>()
                .is_some_and(|image| {
                    image
                        .retained(&self.record.program, self.record.scope)
                        .ok()
                        .flatten()
                        .is_some_and(|record| {
                            Arc::ptr_eq(&record.association, &self.record)
                                && record.physical_availability()
                                    == ManagedIndexSourceAvailability::Available
                        })
                })
    }
}
pub(in crate::mvcc) struct ManagedSourceReadConditions {
    core: Arc<CompanionBranchCellCore>,
    pinned: Arc<CompanionRootImage>,
}
impl ManagedSourceReadConditions {
    pub(in crate::mvcc) fn from_observation(
        observation: &ManagedIndexSourceObservation,
        runtime: u64,
        branch: &BranchId,
        expected_root: u64,
        expected_commit: Option<CommitId>,
    ) -> Result<Self, CompanionPreflightStop> {
        let core = observation
            .core
            .upgrade()
            .ok_or(CompanionPreflightStop::ForeignCell)?;
        if core.runtime_instance_id != runtime || core.branch_id != *branch {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        let current = core
            .current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if current.root_id != expected_root || current.commit_id != expected_commit {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let source = current
            .payload
            .downcast_ref::<ManagedIndexBranchImage>()
            .ok_or(CompanionPreflightStop::ForeignCell)?;
        if !source
            .retained(&observation.record.program, observation.record.scope)
            .map_err(|_| CompanionPreflightStop::ForeignCell)?
            .is_some_and(|actual| Arc::ptr_eq(&actual.association, &observation.record))
        {
            return Err(CompanionPreflightStop::ManagedSourceChanged);
        }
        Ok(Self {
            core: Arc::clone(&core),
            pinned: Arc::clone(&current),
        })
    }
    pub(in crate::mvcc) fn same_reserved_image(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.core, &other.core) && Arc::ptr_eq(&self.pinned, &other.pinned)
    }
    pub(in crate::mvcc) fn matches_reserved(
        &self,
        core: &Arc<CompanionBranchCellCore>,
        image: &Arc<CompanionRootImage>,
    ) -> Result<(), CompanionPreflightStop> {
        if !Arc::ptr_eq(&self.core, core) {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        if !Arc::ptr_eq(&self.pinned, image) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod cache_retirement;

mod rebuild;
pub use rebuild::ManagedIndexSourceRebuild;
