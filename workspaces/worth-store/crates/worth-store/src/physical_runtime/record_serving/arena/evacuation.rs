use super::{ArenaAllocationDenial, SharedArenaAllocationOwner};
use std::sync::Arc;
use worth_store_physical_format::ExtentArenaId;

/// Prevents every allocation lane from refilling an evacuation source. Published
/// free-range truth stays intact until the durable removal publication.
pub(in crate::physical_runtime::record_serving) struct ArenaEvacuationLease {
    owner: SharedArenaAllocationOwner,
    arena: ExtentArenaId,
    forgotten: bool,
    exposed: bool,
}

impl ArenaEvacuationLease {
    pub(in crate::physical_runtime::record_serving) fn acquire(
        owner: &SharedArenaAllocationOwner,
        arena: ExtentArenaId,
    ) -> Result<Self, ArenaAllocationDenial> {
        owner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .begin_evacuation(arena)?;
        Ok(Self {
            owner: Arc::clone(owner),
            arena,
            forgotten: false,
            exposed: false,
        })
    }

    /// The caller must own the durable free-map removal and whole-arena
    /// retirement obligation before discarding the source's allocation state.
    pub(in crate::physical_runtime::record_serving) fn forget_after_publication(mut self) {
        self.owner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .forget_evacuated(self.arena);
        self.forgotten = true;
    }

    /// Once removal intent can escape, abandoned runtime state is not proof
    /// that the source may be allocated again. Reopen resolves that obligation.
    pub(in crate::physical_runtime::record_serving) fn expose_to_retirement(&mut self) {
        self.exposed = true;
    }
}

impl Drop for ArenaEvacuationLease {
    fn drop(&mut self) {
        if !self.forgotten && !self.exposed {
            self.owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .cancel_evacuation(self.arena);
        }
    }
}
