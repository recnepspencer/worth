//! Candidate-bound, callback-free companion installation after Native verifies the head.
use super::{
    CandidateCompanionBinding, CompanionBranchCellCore, CompanionPreflightStop,
    CompanionPublicationCompletionObserver, CompanionRootImage, PublicationCellAdmission,
    PublicationCompanionPreflight,
};
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::{atomic::Ordering, Arc, MutexGuard, TryLockError};

/// Non-cloneable, candidate-bound successor image. Only Relational installs it.
pub struct PreparedPublicationCompanionEffect {
    pub(super) binding: CandidateCompanionBinding,
    pub(super) pinned_topology_generation: u64,
    pub(super) pinned_image: Arc<CompanionRootImage>,
    pub(super) cell: Arc<CompanionBranchCellCore>,
    pub(super) next: Arc<CompanionRootImage>,
    pub(super) retired: Option<Arc<CompanionRootImage>>,
    pub(super) next_topology_generation: u64,
    pub(super) completion: Option<CompanionPublicationCompletionObserver>,
    pub(super) publication_admission: Option<Arc<PublicationCellAdmission>>,
}

impl PreparedPublicationCompanionEffect {
    /// The caller funds this scheduling observer before native publication.
    /// It never changes the authority or selected-image checks of the effect.
    pub fn attach_completion_observer(
        &mut self,
        context: &mut PublicationCompanionPreflight<'_>,
        retained: Arc<dyn Send + Sync>,
    ) -> Result<CompanionPublicationCompletionObserver, CompanionPreflightStop> {
        assert!(
            self.completion.is_none(),
            "one observer per prepared cutover"
        );
        context.claim_work(2)?;
        context.claim_bytes(CompanionPublicationCompletionObserver::retained_bytes())?;
        let observer = CompanionPublicationCompletionObserver::new(retained);
        self.completion = Some(observer.clone());
        Ok(observer)
    }

    pub(crate) fn matches(
        &self,
        binding: &CandidateCompanionBinding,
    ) -> Result<(), CompanionPreflightStop> {
        if self.binding != *binding
            || self.cell.registration_generation != binding.registration_generation
        {
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

    /// Acquire the fixed swap cell before any branch movement.
    pub(crate) fn enter_cutover(
        &mut self,
    ) -> Result<PreparedCompanionCutover<'_>, CompanionPreflightStop> {
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
        Ok(PreparedCompanionCutover {
            cell: &self.cell,
            next: &self.next,
            retired: &mut self.retired,
            next_topology_generation: self.next_topology_generation,
            current,
            completion: self.completion.as_ref(),
            publication_admission: self.publication_admission.as_deref(),
        })
    }
}

pub(crate) struct PreparedCompanionCutover<'a> {
    cell: &'a Arc<CompanionBranchCellCore>,
    next: &'a Arc<CompanionRootImage>,
    retired: &'a mut Option<Arc<CompanionRootImage>>,
    next_topology_generation: u64,
    current: MutexGuard<'a, Arc<CompanionRootImage>>,
    completion: Option<&'a CompanionPublicationCompletionObserver>,
    publication_admission: Option<&'a PublicationCellAdmission>,
}

impl PreparedCompanionCutover<'_> {
    /// No callbacks, allocation, failure, or retired-payload Drop occur here.
    pub(crate) fn install_at(&mut self, position: PatchStreamPosition) {
        self.next.position.store(position.0, Ordering::Release);
        *self.retired = Some(std::mem::replace(&mut *self.current, Arc::clone(self.next)));
        self.cell
            .topology_generation
            .store(self.next_topology_generation, Ordering::Release);
        if let Some(admission) = self.publication_admission {
            admission.installed(self.cell);
        }
        if let Some(completion) = self.completion {
            completion.installed();
        }
    }
}

impl Drop for PreparedPublicationCompanionEffect {
    fn drop(&mut self) {
        if let Some(completion) = &self.completion {
            completion.abort_uninstalled();
        }
        self.cell.reservation_live.store(false, Ordering::Release);
    }
}
