//! Source observations retain real cell/data custody, never an Execution grant.
use super::*;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::{
    ManagedIndexBranchImage, ManagedIndexSourceAssociation, ManagedIndexSourceAvailability,
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use std::sync::Weak;

pub struct ManagedIndexSourceObservation {
    pub(in crate::mvcc) core: Arc<CompanionBranchCellCore>,
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
        let core = self.core.upgrade().ok_or(Denial::RootUnavailable)?;
        if core.registration_generation != registration_generation {
            return Err(Denial::Publication(
                CompanionPreflightStop::RegistrationChanged,
            ));
        }
        if core.runtime_instance_id != selected.runtime_instance_id()
            || core.branch_id != *selected.branch_id()
        {
            return Err(Denial::ForeignSource);
        }
        // A current World product may lawfully select an older Native
        // component (ReuseExact or a pending coordinated successor). Its held
        // Native root names this cache; the latest core is not that authority.
        if self.image.root_id != selected.root_id()
            || self.image.commit_id != selected.commit_id()
            || self.image.position() != selected.position()
        {
            return Err(Denial::ForeignSource);
        }
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
        let core = self.core.upgrade().ok_or(Denial::RootUnavailable)?;
        if core.registration_generation != registration_generation {
            return Err(Denial::Publication(
                CompanionPreflightStop::RegistrationChanged,
            ));
        }
        if core.runtime_instance_id != selected.runtime_instance_id()
            || core.branch_id != *selected.branch_id()
        {
            return Err(Denial::ForeignSource);
        }
        if self.image.root_id != selected.root_id()
            || self.image.commit_id != selected.commit_id()
            || self.image.position() != selected.position()
        {
            return Err(Denial::ForeignSource);
        }
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
            core,
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
        Arc::ptr_eq(&self.core, &other.core) && Arc::ptr_eq(&self.record, &other.record)
    }

    /// Compare the complete retained Source support at an actual Native image.
    /// This is currentness only; it does not admit a principal or Source READ.
    pub(crate) fn remains_current_at(
        &self,
        selected: &PositionedRelationalSnapshot,
        registration_generation: u64,
    ) -> bool {
        if self.core.runtime_instance_id != selected.runtime_instance_id()
            || self.core.branch_id != *selected.branch_id()
            || self.core.registration_generation != registration_generation
        {
            return false;
        }
        let current = self
            .core
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
        if observation.core.runtime_instance_id != runtime || observation.core.branch_id != *branch
        {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        let current = observation
            .core
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
            core: Arc::clone(&observation.core),
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
