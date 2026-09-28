use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::{
    arena::{self, SharedArenaAllocationOwner},
    AdmittedRecordPlacementPolicy, RecordAppendDenial, RecordAppendError,
};

impl RecordPublicationDirector {
    pub(super) fn arena_allocation_owner(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        placement: AdmittedRecordPlacementPolicy,
        current: &worth_store_physical_format::DurableFreeSpaceManifestHeader,
    ) -> Result<SharedArenaAllocationOwner, RecordAppendError> {
        let runtime = self.runtime.upgrade().ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublicationAuthorityReleased,
        ))?;
        if arena::qualified_arena_alignment(runtime.executor.record_serving_media())
            != Some(current.arena_alignment())
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PlacementFormatMismatch,
            ));
        }
        let mut preparation = self.preparation.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(owner) = &preparation.arenas {
            if owner.lock().unwrap_or_else(|e| e.into_inner()).capacity()
                != placement.arena_capacity()
            {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::PlacementFormatMismatch,
                ));
            }
            return Ok(std::sync::Arc::clone(owner));
        }
        let owner = arena::reconstruct(
            allocation,
            self.residency.clone(),
            self.format,
            self.access,
            placement,
            current,
        )?;
        if let Some(arena) = preparation.recovered_retiring_arena {
            owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .begin_evacuation(arena)
                .map_err(|_| {
                    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
                })?;
        }
        if let Some(claim) = &preparation.recovered_copy_destination {
            claim
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .restore(&owner)
                .map_err(|denial| {
                    RecordAppendError::Denied(match denial {
                        arena::ArenaAllocationDenial::RangeBudget => {
                            RecordAppendDenial::PhysicalPressure
                        }
                        _ => RecordAppendDenial::PublishedLayoutDamaged,
                    })
                })?;
        }
        preparation.arenas = Some(std::sync::Arc::clone(&owner));
        Ok(owner)
    }
}
