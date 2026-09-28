use super::*;
use crate::physical_runtime::record_serving::arena::{
    ArenaAllocationDenial, SharedArenaAllocationOwner,
};
use worth_store_physical_format::{ExtentArenaRange, PhysicalExtentCopyResolutionKind};

pub(in crate::physical_runtime::record_serving::publication::director) type SharedCopyDestination =
    Arc<Mutex<CopyDestination>>;

pub(in crate::physical_runtime::record_serving::publication::director) enum CopyDestination {
    Reserved(ArenaReservationObligation),
    /// Installed before serving; the first allocator reconstruction subtracts
    /// this exact range before returning the allocator to any caller.
    Deferred(ExtentArenaRange),
    Published,
    Released,
}

impl CopyDestination {
    pub(in crate::physical_runtime::record_serving::publication::director) fn restore(
        &mut self,
        owner: &SharedArenaAllocationOwner,
    ) -> Result<(), ArenaAllocationDenial> {
        if let Self::Deferred(range) = self {
            *self = Self::Reserved(ArenaReservationObligation::restore(owner, *range)?);
        }
        Ok(())
    }
    pub(in crate::physical_runtime::record_serving::publication::director::extent_copy) fn cancel_after_resolution(
        &mut self,
    ) -> Result<(), ()> {
        match self {
            Self::Reserved(claim) => claim.cancel_after_resolution()?,
            Self::Deferred(_) => {}
            Self::Published | Self::Released => return Err(()),
        }
        *self = Self::Released;
        Ok(())
    }
}

/// Persisted verified evidence is deliberately distinct from a live I/O
/// receipt. Neither variant permits forging fresh durability work.
pub(in crate::physical_runtime::record_serving::publication::director::extent_copy) enum CopyResolution
{
    Live(
        PhysicalExtentCopyResolutionKind,
        crate::physical_runtime::durability::DurableMaintenanceReceipt,
    ),
    Recovered {
        kind: PhysicalExtentCopyResolutionKind,
        end_lsn: u64,
    },
}
impl CopyResolution {
    pub(in crate::physical_runtime::record_serving::publication::director::extent_copy) fn kind(
        &self,
    ) -> PhysicalExtentCopyResolutionKind {
        match self {
            Self::Live(kind, _) | Self::Recovered { kind, .. } => *kind,
        }
    }
    pub(in crate::physical_runtime::record_serving::publication::director::extent_copy) fn end_lsn(
        &self,
    ) -> u64 {
        match self {
            Self::Live(_, receipt) => receipt.interval().3,
            Self::Recovered { end_lsn, .. } => *end_lsn,
        }
    }
    pub(in crate::physical_runtime::record_serving::publication::director::extent_copy) fn start_lsn(
        &self,
    ) -> u64 {
        match self {
            Self::Live(_, receipt) => receipt.interval().2,
            Self::Recovered { end_lsn, .. } => end_lsn - 1,
        }
    }
}
