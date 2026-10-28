use crate::identity::CompositeCommitIdentity;

use super::{ProductBranchHeadProtection, ProductBranchReferenceCell};
use crate::branch::{ProductBranchObservation, RuntimeWorldBranchReplacementDenial as Denial};
use std::sync::{Arc, TryLockError};

/// The exact product-head custody released by one cell tombstone.
///
/// Keeping the released protection in this value lets the registry finish its
/// index transition before any retention destructor runs.
pub(crate) struct ProductBranchReferenceRetirement {
    retired_head: CompositeCommitIdentity,
    released_protection: Option<ProductBranchHeadProtection>,
}

impl ProductBranchReferenceRetirement {
    pub(crate) fn into_parts(
        self,
    ) -> (CompositeCommitIdentity, Option<ProductBranchHeadProtection>) {
        (self.retired_head, self.released_protection)
    }
}

impl ProductBranchReferenceCell {
    /// The registry guard precedes both cell guards. No callback can acquire
    /// another owner while these guards are held; no field is moved on refusal.
    pub(crate) fn try_retire_pair(
        &self,
        old: &ProductBranchObservation,
        candidate: &Self,
        expected_candidate: &ProductBranchObservation,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<ProductBranchReferenceRetirement, Denial> {
        if Arc::ptr_eq(&self.state.current, &candidate.state.current) {
            return Err(Denial::AliasedCells);
        }
        let (mut old_image, candidate_image) =
            if old.branch_identity() < expected_candidate.branch_identity() {
                let old_image = try_image(self)?;
                let candidate_image = try_image(candidate)?;
                (old_image, candidate_image)
            } else {
                let candidate_image = try_image(candidate)?;
                let old_image = try_image(self)?;
                (old_image, candidate_image)
            };
        if old_image.protection.is_none() {
            return Err(Denial::OldRetired);
        }
        if candidate_image.protection.is_none() {
            return Err(Denial::CandidateRetired);
        }
        if &old_image.snapshot != old.snapshot() {
            return Err(Denial::OldHeadMoved);
        }
        if &candidate_image.snapshot != expected_candidate.snapshot() {
            return Err(Denial::CandidateHeadMoved);
        }
        // Consult reads the actual original stop, including exhausted active
        // work, without allocation or opening a scope under either cell lock.
        execution.consult().map_err(Denial::Request)?;
        // Composite occurrence identity is fixed-size. Both full snapshots
        // remain guarded until this exact old image loses its protection.
        let retired_head = old.selected_commit().clone();
        Ok(retire_image(&mut old_image, retired_head))
    }

    /// Stop every future movement through this cell and capture its exact final
    /// commit at the same linearization point. The identity clone is prepared
    /// before product-head custody moves out of the active image.
    pub(crate) fn retire(&self) -> ProductBranchReferenceRetirement {
        let mut current = self.state.write();
        let retired_head = current.snapshot.selected_commit().clone();
        retire_image(&mut current, retired_head)
    }
}

fn try_image(
    cell: &ProductBranchReferenceCell,
) -> Result<std::sync::RwLockWriteGuard<'_, super::ProductBranchReferenceImage>, Denial> {
    match cell.state.current.try_write() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
        Err(TryLockError::WouldBlock) => Err(Denial::CellBusy),
    }
}

fn retire_image(
    current: &mut super::ProductBranchReferenceImage,
    retired_head: CompositeCommitIdentity,
) -> ProductBranchReferenceRetirement {
    let mut released_protection = current.protection.take();
    if let Some(protection) = released_protection.as_mut() {
        protection.seat_history(false);
    }
    ProductBranchReferenceRetirement {
        retired_head,
        released_protection,
    }
}
