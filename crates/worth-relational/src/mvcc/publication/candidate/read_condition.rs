//! Carry one native companion read condition to the reserved publication image.
use super::*;
use crate::mvcc::{CompanionBranchCell, CompanionBranchImage, CompanionPreflightStop};

impl PreparedRelationalCommitCandidate {
    /// Restrict this prepared candidate to the exact native image observed by
    /// its caller. This grants no publication authority. Preflight must seal
    /// the same registered cell and image under its existing reservation;
    /// omitted, foreign or changed images refuse before irreversible cutover.
    /// The condition is move-only and is never checkpoint or replay material.
    pub fn require_companion_image<T: Send + Sync + 'static>(
        &mut self,
        cell: &CompanionBranchCell<T>,
        image: CompanionBranchImage<T>,
    ) -> Result<(), CompanionPreflightStop> {
        let mut payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let payload = payload
            .as_mut()
            .ok_or(CompanionPreflightStop::RegistrationChanged)?;
        if payload.companion_read.is_some() {
            return Err(CompanionPreflightStop::RegistrationChanged);
        }
        let condition = cell.read_condition(image);
        if !condition.admits_basis(
            self.runtime_instance_id,
            &self.branch_id,
            payload.expected_root.id(),
            payload.expected_root.commit_id(),
        ) {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        payload.companion_read = Some(condition);
        Ok(())
    }
}
