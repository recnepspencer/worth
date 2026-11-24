//! Source observations retain selected data/affinity, never newer live cache custody.
use super::*;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::{
    ManagedIndexBranchImage, ManagedIndexSourceAvailability,
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
#[cfg(test)]
use std::sync::Weak;

pub use crate::branch::projection_view::managed_source::ManagedIndexSourceObservation;
pub(crate) use crate::branch::projection_view::managed_source::ManagedSourceRootImage;

#[cfg(test)]
#[path = "managed_source/correspondence/tests.rs"]
mod correspondence_tests;

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
            .serving_cell()
            .ok_or(CompanionPreflightStop::ForeignCell)?;
        if core.runtime_instance_id() != runtime || core.branch_id() != branch {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        let current = core.read_image();
        let current = current.image();
        if current.root_id() != expected_root || current.commit_id() != expected_commit {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let source = current
            .payload::<ManagedIndexBranchImage>()
            .ok_or(CompanionPreflightStop::ForeignCell)?;
        if !source
            .retained(
                &observation.association().program,
                observation.association().scope,
            )
            .map_err(|_| CompanionPreflightStop::ForeignCell)?
            .is_some_and(|actual| Arc::ptr_eq(&actual.association, observation.association()))
        {
            return Err(CompanionPreflightStop::ManagedSourceChanged);
        }
        Ok(Self {
            core: Arc::clone(&core),
            pinned: Arc::clone(current),
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
