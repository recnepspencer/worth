use crate::identity::CompositeCommitIdentity;

use super::{
    ProductBranchHeadProtection, ProductBranchReferenceCell, ProductBranchReferenceCellDenial,
    ProductBranchReferenceImage, ProductBranchReferenceSnapshot, ProductMovementPhase,
};

impl ProductBranchReferenceImage {
    pub(super) fn is_retired_or_retiring(&self) -> bool {
        self.protection.is_none() || self.movement.blocks_live_observation()
    }
}

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

/// Exclusive cell barrier held while a durable `BranchRetired` record is
/// published. Proven no effect clears it; unresolved or unwind keeps a
/// recovery-required barrier so no further movement can race the journal.
pub(crate) struct WorldProductRetirementPermit {
    cell: ProductBranchReferenceCell,
    armed: bool,
}

impl WorldProductRetirementPermit {
    pub(crate) fn proven_no_effect(mut self) {
        let mut current = self.cell.state.write();
        if matches!(current.movement, ProductMovementPhase::RetirementInFlight) {
            current.movement = ProductMovementPhase::Idle;
        }
        self.armed = false;
    }

    pub(crate) fn finalize_retirement(mut self) -> ProductBranchReferenceRetirement {
        let mut current = self.cell.state.write();
        assert!(
            matches!(current.movement, ProductMovementPhase::RetirementInFlight),
            "durable retirement barrier must remain installed until finalization"
        );
        current.movement = ProductMovementPhase::Idle;
        let retired_head = current.snapshot.selected_commit().clone();
        let mut released_protection = current.protection.take();
        if let Some(protection) = released_protection.as_mut() {
            protection.seat_history(false);
        }
        self.armed = false;
        ProductBranchReferenceRetirement {
            retired_head,
            released_protection,
        }
    }
}

impl Drop for WorldProductRetirementPermit {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut current = self.cell.state.write();
        if matches!(current.movement, ProductMovementPhase::RetirementInFlight) {
            current.movement = ProductMovementPhase::RetirementRecoveryRequired;
        }
    }
}

impl ProductBranchReferenceCell {
    pub(crate) fn is_retired_or_retiring(&self) -> bool {
        self.state.read().is_retired_or_retiring()
    }

    /// Stop every future movement through this cell and capture its exact final
    /// commit at the same linearization point. The identity clone is prepared
    /// before product-head custody moves out of the active image.
    pub(crate) fn retire(
        &self,
    ) -> Result<ProductBranchReferenceRetirement, ProductBranchReferenceCellDenial> {
        let mut current = self.state.write();
        if let Some(denial) = current.movement.denial() {
            return Err(denial);
        }
        let retired_head = current.snapshot.selected_commit().clone();
        let mut released_protection = current.protection.take();
        if let Some(protection) = released_protection.as_mut() {
            protection.seat_history(false);
        }
        Ok(ProductBranchReferenceRetirement {
            retired_head,
            released_protection,
        })
    }

    /// Block all future movements on this cell and return its exact current
    /// head snapshot for durable `BranchRetired` publication.
    pub(crate) fn begin_durable_retirement(
        &self,
    ) -> Result<(WorldProductRetirementPermit, ProductBranchReferenceSnapshot), ProductBranchReferenceCellDenial>
    {
        let mut current = self.state.write();
        if current.protection.is_none() {
            return Err(ProductBranchReferenceCellDenial::Retired);
        }
        if let Some(denial) = current.movement.denial() {
            return Err(denial);
        }
        let snapshot = current.snapshot.clone();
        current.movement = ProductMovementPhase::RetirementInFlight;
        Ok((
            WorldProductRetirementPermit {
                cell: self.clone(),
                armed: true,
            },
            snapshot,
        ))
    }
}

#[cfg(test)]
#[path = "retirement_tests.rs"]
mod tests;
