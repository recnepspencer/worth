use std::sync::Arc;

use super::CompanionBranchCell;
use super::CompanionBranchImage;
use worth_relational_branch::internal_port::branch::projection_view::companion_slot::{
    CompanionBranchCellCore, CompanionCellEditStop, CompanionRootImage,
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
        let Some(current) = self.core.try_read_image() else {
            return Err(CompanionDerivedRootPreparationStop::Cell {
                reason: CompanionCellEditStop::PreflightPending,
                expected,
                payload,
            });
        };
        let generation = self.core.generation();
        let matched = Arc::ptr_eq(current.image(), &expected.image)
            && generation == expected.topology_generation
            && expected.root_id == expected.image.root_id()
            && expected.commit_id == expected.image.commit_id()
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
    let next = CompanionRootImage::new(
        expected.root_id,
        expected.commit_id,
        expected.position,
        Arc::clone(&payload),
    );
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
        let result = self.core.replace_derived(
            &self.expected.image,
            self.expected.topology_generation,
            &self.next,
        );
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
}
