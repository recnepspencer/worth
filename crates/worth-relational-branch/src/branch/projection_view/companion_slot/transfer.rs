//! The held cell reservation moves its one selected image after native cutover.
use super::{CompanionBranchCellCore, CompanionRootImage, CompanionSlotReservation};
use crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop;
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::{atomic::Ordering, Arc, MutexGuard};

pub struct PreparedCompanionSlotSwap {
    reservation: CompanionSlotReservation,
    pinned: Arc<CompanionRootImage>,
    next: Arc<CompanionRootImage>,
    retired: Option<Arc<CompanionRootImage>>,
    pinned_generation: u64,
    next_generation: u64,
}

impl PreparedCompanionSlotSwap {
    pub fn prepare<T: Send + Sync + 'static>(
        reservation: CompanionSlotReservation,
        pinned: Arc<CompanionRootImage>,
        next_root_id: u64,
        next_commit_id: crate::history::data::coordinates::CommitId,
        payload: Arc<T>,
        pinned_generation: u64,
    ) -> Result<Self, CompanionPreflightStop> {
        if reservation.cell.generation() != pinned_generation {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        let next_generation = pinned_generation
            .checked_add(1)
            .ok_or(CompanionPreflightStop::RegistrationChanged)?;
        Ok(Self {
            reservation,
            pinned,
            next: CompanionRootImage::new(next_root_id, Some(next_commit_id), None, payload),
            retired: None,
            pinned_generation,
            next_generation,
        })
    }

    pub fn cell(&self) -> &Arc<CompanionBranchCellCore> {
        &self.reservation.cell
    }

    pub fn pinned(&self) -> &Arc<CompanionRootImage> {
        &self.pinned
    }

    pub fn retain_source_root(
        &self,
    ) -> Option<super::super::managed_source::ManagedSourceRootImage> {
        super::super::managed_source::ManagedSourceRootImage::retain_image(self.cell(), &self.next)
    }

    pub fn check(&self, registration: u64) -> Result<(), CompanionPreflightStop> {
        let cell = self.cell();
        if cell.registration_generation() != registration {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        if !cell.reservation_live.load(Ordering::Acquire)
            || cell.generation() != self.pinned_generation
        {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(())
    }

    pub fn enter(&mut self) -> Result<AdmittedCompanionSlotSwap<'_>, CompanionPreflightStop> {
        let cell = &self.reservation.cell;
        let current = cell
            .try_read_image()
            .ok_or(CompanionPreflightStop::TopologyPending)?;
        if cell.generation() != self.pinned_generation
            || !Arc::ptr_eq(current.image(), &self.pinned)
        {
            return Err(CompanionPreflightStop::TopologyPending);
        }
        Ok(AdmittedCompanionSlotSwap {
            cell,
            next: &self.next,
            retired: &mut self.retired,
            next_generation: self.next_generation,
            current: current.0,
        })
    }
}

pub struct AdmittedCompanionSlotSwap<'a> {
    cell: &'a Arc<CompanionBranchCellCore>,
    next: &'a Arc<CompanionRootImage>,
    retired: &'a mut Option<Arc<CompanionRootImage>>,
    next_generation: u64,
    current: MutexGuard<'a, Arc<CompanionRootImage>>,
}

impl AdmittedCompanionSlotSwap<'_> {
    pub fn cell(&self) -> &Arc<CompanionBranchCellCore> {
        self.cell
    }

    pub fn install_at(&mut self, position: PatchStreamPosition) {
        self.next.position.store(position.0, Ordering::Release);
        *self.retired = Some(std::mem::replace(&mut *self.current, Arc::clone(self.next)));
        self.cell
            .topology_generation
            .store(self.next_generation, Ordering::Release);
    }
}
