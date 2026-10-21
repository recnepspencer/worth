use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, TryLockError};

use super::{
    CompanionBranchCell, CompanionBranchCellCore, CompanionBranchImage, CompanionCellEditStop,
    CompanionRootImage, DerivedCellEdit,
};

pub enum CompanionDerivedRootPreparationStop<T: Send + Sync + 'static> {
    Cell {
        reason: CompanionCellEditStop,
        expected: CompanionBranchImage<T>,
        payload: Arc<T>,
    },
}

pub struct PreparedCompanionDerivedRoot<T: Send + Sync + 'static> {
    core: Arc<CompanionBranchCellCore>,
    expected: CompanionBranchImage<T>,
    payload: Arc<T>,
    next: Arc<CompanionRootImage>,
}

pub struct CompanionDerivedRootStopped<T: Send + Sync + 'static> {
    reason: CompanionCellEditStop,
    prepared: PreparedCompanionDerivedRoot<T>,
}

impl<T: Send + Sync + 'static> CompanionDerivedRootStopped<T> {
    pub const fn reason(&self) -> CompanionCellEditStop {
        self.reason
    }

    pub fn into_parts(self) -> (CompanionCellEditStop, PreparedCompanionDerivedRoot<T>) {
        (self.reason, self.prepared)
    }
}

/// Drop this after the caller's outer Product/lineage guard. Last-owner payload
/// and registration cleanup never runs in the native image mutex section.
pub struct CompanionDerivedRootCleanup<T: Send + Sync + 'static> {
    _retired: Arc<CompanionRootImage>,
    _expected: CompanionBranchImage<T>,
    _core: Arc<CompanionBranchCellCore>,
}

pub struct CompanionDerivedRootInstalled<T: Send + Sync + 'static> {
    image: CompanionBranchImage<T>,
    cleanup: CompanionDerivedRootCleanup<T>,
}

impl<T: Send + Sync + 'static> CompanionDerivedRootInstalled<T> {
    pub fn into_parts(self) -> (CompanionBranchImage<T>, CompanionDerivedRootCleanup<T>) {
        (self.image, self.cleanup)
    }
}

impl<T: Send + Sync + 'static> CompanionBranchCell<T> {
    pub fn prepare_derived_root_at_same_position(
        &self,
        expected: CompanionBranchImage<T>,
        payload: Arc<T>,
    ) -> Result<PreparedCompanionDerivedRoot<T>, CompanionDerivedRootPreparationStop<T>> {
        let current = match self.core.current.try_lock() {
            Ok(current) => current,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => {
                return Err(CompanionDerivedRootPreparationStop::Cell {
                    reason: CompanionCellEditStop::PreflightPending,
                    expected,
                    payload,
                });
            }
        };
        let generation = self.core.topology_generation.load(Ordering::Acquire);
        let matched = Arc::ptr_eq(&*current, &expected.image)
            && generation == expected.topology_generation
            && expected.root_id == expected.image.root_id
            && expected.commit_id == expected.image.commit_id
            && expected.position == expected.image.position();
        drop(current);
        if !matched {
            return Err(CompanionDerivedRootPreparationStop::Cell {
                reason: CompanionCellEditStop::TopologyGenerationChanged,
                expected,
                payload,
            });
        }
        Ok(prepare_admitted(self, expected, payload))
    }
}

pub(super) fn prepare_admitted<T: Send + Sync + 'static>(
    cell: &CompanionBranchCell<T>,
    expected: CompanionBranchImage<T>,
    payload: Arc<T>,
) -> PreparedCompanionDerivedRoot<T> {
    let next = Arc::new(CompanionRootImage {
        root_id: expected.root_id,
        commit_id: expected.commit_id,
        position: AtomicU64::new(expected.position.map_or(0, |position| position.0)),
        payload: Arc::clone(&payload) as Arc<dyn Any + Send + Sync>,
    });
    PreparedCompanionDerivedRoot {
        core: Arc::clone(&cell.core),
        expected,
        payload,
        next,
    }
}

impl<T: Send + Sync + 'static> PreparedCompanionDerivedRoot<T> {
    pub fn install(
        self,
    ) -> Result<CompanionDerivedRootInstalled<T>, CompanionDerivedRootStopped<T>> {
        if self
            .core
            .reservation_live
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(CompanionDerivedRootStopped {
                reason: CompanionCellEditStop::PreflightPending,
                prepared: self,
            });
        }
        let gate = DerivedCellEdit(&self.core);
        let result = self.install_with_gate();
        drop(gate);
        let (retired, next_generation) = match result {
            Ok(installed) => installed,
            Err(reason) => {
                return Err(CompanionDerivedRootStopped {
                    reason,
                    prepared: self,
                });
            }
        };
        let image = CompanionBranchImage {
            image: self.next,
            payload: self.payload,
            root_id: self.expected.root_id,
            commit_id: self.expected.commit_id,
            position: self.expected.position,
            topology_generation: next_generation,
        };
        Ok(CompanionDerivedRootInstalled {
            image,
            cleanup: CompanionDerivedRootCleanup {
                _retired: retired,
                _expected: self.expected,
                _core: self.core,
            },
        })
    }

    fn install_with_gate(&self) -> Result<(Arc<CompanionRootImage>, u64), CompanionCellEditStop> {
        let mut current = match self.core.current.try_lock() {
            Ok(current) => current,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(CompanionCellEditStop::PreflightPending),
        };
        let generation = self.core.topology_generation.load(Ordering::Acquire);
        if generation != self.expected.topology_generation
            || !Arc::ptr_eq(&*current, &self.expected.image)
            || self.expected.root_id != self.next.root_id
            || self.expected.commit_id != self.next.commit_id
            || self.expected.position != self.next.position()
        {
            return Err(CompanionCellEditStop::TopologyGenerationChanged);
        }
        let next_generation = generation
            .checked_add(1)
            .ok_or(CompanionCellEditStop::TopologyGenerationExhausted)?;
        let retired = std::mem::replace(&mut *current, Arc::clone(&self.next));
        self.core
            .topology_generation
            .store(next_generation, Ordering::Release);
        drop(current);
        Ok((retired, next_generation))
    }
}
