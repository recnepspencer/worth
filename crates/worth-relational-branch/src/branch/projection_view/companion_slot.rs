//! Physical companion image and the exact native cell that retains it.
use crate::branch::managed_source_affinity::ManagedSourceCellAffinity;
use crate::history::data::coordinates::{BranchId, CommitId};
use crate::publication::patch::data::PatchStreamPosition;
use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError};

pub mod affinity;
pub mod transfer;

pub struct CompanionRootImage {
    root_id: u64,
    commit_id: Option<CommitId>,
    position: AtomicU64,
    payload: Arc<dyn Any + Send + Sync>,
}

impl CompanionRootImage {
    pub fn new<T: Send + Sync + 'static>(
        root_id: u64,
        commit_id: Option<CommitId>,
        position: Option<PatchStreamPosition>,
        payload: Arc<T>,
    ) -> Arc<Self> {
        Arc::new(Self {
            root_id,
            commit_id,
            position: AtomicU64::new(position.map_or(0, |position| position.0)),
            payload,
        })
    }

    pub const fn root_id(&self) -> u64 {
        self.root_id
    }

    pub const fn commit_id(&self) -> Option<CommitId> {
        self.commit_id
    }

    pub fn position(&self) -> Option<PatchStreamPosition> {
        match self.position.load(Ordering::Acquire) {
            0 => None,
            value => Some(PatchStreamPosition(value)),
        }
    }

    pub fn payload<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.payload.downcast_ref()
    }

    pub fn retain_payload<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        Arc::clone(&self.payload).downcast().ok()
    }
}

pub struct CompanionBranchCellCore {
    runtime_instance_id: u64,
    registration_generation: u64,
    branch_id: BranchId,
    current: Mutex<Arc<CompanionRootImage>>,
    reservation_live: AtomicBool,
    topology_generation: AtomicU64,
    source_affinity: OnceLock<Arc<ManagedSourceCellAffinity>>,
    _placement_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl CompanionBranchCellCore {
    pub fn new(
        runtime_instance_id: u64,
        branch_id: BranchId,
        registration_generation: u64,
        image: Arc<CompanionRootImage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime_instance_id,
            registration_generation,
            branch_id,
            current: Mutex::new(image),
            reservation_live: AtomicBool::new(false),
            topology_generation: AtomicU64::new(1),
            source_affinity: OnceLock::new(),
            _placement_memory: None,
        })
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
    pub const fn registration_generation(&self) -> u64 {
        self.registration_generation
    }
    pub const fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn generation(&self) -> u64 {
        self.topology_generation.load(Ordering::Acquire)
    }

    pub fn read_image(&self) -> CompanionSlotRead<'_> {
        CompanionSlotRead(
            self.current
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    pub fn try_read_image(&self) -> Option<CompanionSlotRead<'_>> {
        match self.current.try_lock() {
            Ok(guard) => Some(CompanionSlotRead(guard)),
            Err(TryLockError::Poisoned(poisoned)) => Some(CompanionSlotRead(poisoned.into_inner())),
            Err(TryLockError::WouldBlock) => None,
        }
    }

    pub(crate) fn try_lock_unpoisoned_image(&self) -> Option<CompanionSlotRead<'_>> {
        self.current.try_lock().ok().map(CompanionSlotRead)
    }

    pub(crate) fn lock_unpoisoned_image(&self) -> CompanionSlotRead<'_> {
        CompanionSlotRead(self.current.lock().unwrap())
    }

    fn reserve(&self) -> bool {
        self.reservation_live
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub fn reserve_for_publication(self: &Arc<Self>) -> Option<CompanionSlotReservation> {
        self.reserve().then(|| CompanionSlotReservation {
            cell: Arc::clone(self),
        })
    }

    pub fn bind_unshared_placement_funding(
        owner: &mut Arc<Self>,
        funding: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
    ) {
        Arc::get_mut(owner)
            .expect("fresh unshared placement core")
            ._placement_memory = funding;
    }

    pub fn placement_funding(&self) -> Option<&Arc<worth_execution::ExecutionMemoryReservation>> {
        self._placement_memory.as_ref()
    }

    pub fn source_affinity(&self) -> Option<&Arc<ManagedSourceCellAffinity>> {
        self.source_affinity.get()
    }

    pub fn replace_derived(
        &self,
        pinned: &Arc<CompanionRootImage>,
        pinned_generation: u64,
        next: &Arc<CompanionRootImage>,
    ) -> Result<(Arc<CompanionRootImage>, u64), CompanionCellEditStop> {
        if !self.reserve() {
            return Err(CompanionCellEditStop::PreflightPending);
        }
        let _edit = DerivedCellEdit(self);
        let mut current = self
            .try_read_image()
            .ok_or(CompanionCellEditStop::PreflightPending)?;
        let generation = self.generation();
        if generation != pinned_generation
            || !Arc::ptr_eq(current.image(), pinned)
            || pinned.root_id() != next.root_id()
            || pinned.commit_id() != next.commit_id()
            || pinned.position() != next.position()
        {
            return Err(CompanionCellEditStop::TopologyGenerationChanged);
        }
        let next_generation = generation
            .checked_add(1)
            .ok_or(CompanionCellEditStop::TopologyGenerationExhausted)?;
        let retired = std::mem::replace(&mut *current.0, Arc::clone(next));
        self.topology_generation
            .store(next_generation, Ordering::Release);
        drop(current);
        Ok((retired, next_generation))
    }
}

pub struct CompanionSlotRead<'a>(MutexGuard<'a, Arc<CompanionRootImage>>);

impl CompanionSlotRead<'_> {
    pub fn image(&self) -> &Arc<CompanionRootImage> {
        &self.0
    }
}

struct DerivedCellEdit<'a>(&'a CompanionBranchCellCore);

impl Drop for DerivedCellEdit<'_> {
    fn drop(&mut self) {
        self.0.reservation_live.store(false, Ordering::Release);
    }
}

pub struct CompanionSlotReservation {
    cell: Arc<CompanionBranchCellCore>,
}

impl CompanionSlotReservation {
    pub fn cell(&self) -> &Arc<CompanionBranchCellCore> {
        &self.cell
    }
}

impl Drop for CompanionSlotReservation {
    fn drop(&mut self) {
        self.cell.reservation_live.store(false, Ordering::Release);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionCellEditStop {
    PreflightPending,
    TopologyGenerationChanged,
    TopologyGenerationExhausted,
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait CompanionBranchCellCoreFixturePort: Sized {
    fn lock_unpoisoned_image_for_test(&self) -> CompanionSlotRead<'_>;
    fn try_lock_unpoisoned_image_for_test(&self) -> Option<CompanionSlotRead<'_>>;
}

impl CompanionBranchCellCoreFixturePort for CompanionBranchCellCore {
    fn lock_unpoisoned_image_for_test(&self) -> CompanionSlotRead<'_> {
        self.lock_unpoisoned_image()
    }

    fn try_lock_unpoisoned_image_for_test(&self) -> Option<CompanionSlotRead<'_>> {
        self.try_lock_unpoisoned_image()
    }
}
