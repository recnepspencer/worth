//! One reserved immutable cell replacement; compound cutover fixes its arity.
use super::{
    CompanionBranchCellCore, CompanionPreflightStop, CompanionRootImage, PublicationCellAdmission,
};
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::{atomic::Ordering, Arc, MutexGuard, TryLockError};

pub(super) struct PreparedCellSwap {
    pub(super) pinned_topology_generation: u64,
    pub(super) pinned_image: Arc<CompanionRootImage>,
    pub(super) cell: Arc<CompanionBranchCellCore>,
    pub(super) next: Arc<CompanionRootImage>,
    pub(super) retired: Option<Arc<CompanionRootImage>>,
    pub(super) next_topology_generation: u64,
    pub(super) publication_admission: Option<Arc<PublicationCellAdmission>>,
}

impl PreparedCellSwap {
    pub(super) fn check(&self, registration_generation: u64) -> Result<(), CompanionPreflightStop> {
        if self.cell.registration_generation != registration_generation {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        if !self.cell.reservation_live.load(Ordering::Acquire)
            || self.cell.topology_generation.load(Ordering::Acquire)
                != self.pinned_topology_generation
            || self.pinned_topology_generation.checked_add(1) != Some(self.next_topology_generation)
        {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(())
    }

    pub(super) fn enter(&mut self) -> Result<PreparedCellCutover<'_>, CompanionPreflightStop> {
        let current = match self.cell.current.try_lock() {
            Ok(current) => current,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(CompanionPreflightStop::TopologyPending),
        };
        if self.cell.topology_generation.load(Ordering::Acquire) != self.pinned_topology_generation
            || !Arc::ptr_eq(&*current, &self.pinned_image)
        {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(PreparedCellCutover {
            cell: &self.cell,
            next: &self.next,
            retired: &mut self.retired,
            next_topology_generation: self.next_topology_generation,
            current,
            publication_admission: self.publication_admission.as_deref(),
        })
    }
}

impl Drop for PreparedCellSwap {
    fn drop(&mut self) {
        self.cell.reservation_live.store(false, Ordering::Release);
    }
}

pub(super) struct PreparedCellCutover<'a> {
    cell: &'a Arc<CompanionBranchCellCore>,
    next: &'a Arc<CompanionRootImage>,
    retired: &'a mut Option<Arc<CompanionRootImage>>,
    next_topology_generation: u64,
    current: MutexGuard<'a, Arc<CompanionRootImage>>,
    publication_admission: Option<&'a PublicationCellAdmission>,
}

impl PreparedCellCutover<'_> {
    pub(super) fn install_at(&mut self, position: PatchStreamPosition) {
        self.next.position.store(position.0, Ordering::Release);
        *self.retired = Some(std::mem::replace(&mut *self.current, Arc::clone(self.next)));
        self.cell
            .topology_generation
            .store(self.next_topology_generation, Ordering::Release);
        if let Some(admission) = self.publication_admission {
            admission.installed(self.cell);
        }
    }
}
