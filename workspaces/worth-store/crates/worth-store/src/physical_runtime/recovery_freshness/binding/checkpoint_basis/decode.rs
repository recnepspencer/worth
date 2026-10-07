//! Temporary decoded owners are disposed before their native Recovery grant.

use std::{alloc::Layout, sync::atomic::AtomicUsize};

use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::durability::{
    DecodedPhysicalMutationBindingRecord, PhysicalBindingCompactionRecordDecodeDenial,
    PhysicalBindingDecodingContext, PhysicalPersistedBindingDecodeDenial,
};
use crate::physical_runtime::{
    CompletedPhysicalMutationFact, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentDenial,
};

use super::super::{checkpoint_evidence, StoreRecoveryOperationEvidence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreRecoveryCheckpointBindingAllocationDenial {
    StoreMismatch,
    PoolMismatch,
    BackingMismatch,
    SizeOverflow,
    Backing {
        requested: u64,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
}

/// This upper bound uses the actual payload, not the maximum checkpoint budget.
/// Canonical comparison has no storage. A nested persisted payload and completed
/// record storage each include their possible Vec-to-Box conversion overlap.
pub(in crate::physical_runtime) fn checkpoint_binding_decode_peak(
    payload_len: usize,
) -> Option<u64> {
    let payload = u64::try_from(payload_len).ok()?;
    let records =
        (payload / 32).checked_mul(core::mem::size_of::<PersistedRecordIdentity>() as u64)?;
    let (arc, _) = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<CompletedPhysicalMutationFact>())
        .ok()?;
    payload
        .checked_mul(2)?
        .checked_add(records.checked_mul(2)?)?
        .checked_add(u64::try_from(arc.pad_to_align().size()).ok()?)
}

/// Only inline evidence leaves this scope. No decoded Box, Vec or Arc can
/// outlive the distinct grant acquired before entering the semantic decoder.
pub(in crate::physical_runtime) fn decode_checkpoint_evidence(
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    payload: &[u8],
    context: PhysicalBindingDecodingContext,
    selected_generation: u64,
) -> Result<Option<StoreRecoveryOperationEvidence>, StoreRecoveryCheckpointBindingAllocationDenial>
{
    use StoreRecoveryCheckpointBindingAllocationDenial as Denial;
    if window.store_identity() != context.store_identity() {
        return Err(Denial::StoreMismatch);
    }
    let requested = checkpoint_binding_decode_peak(payload.len()).ok_or(Denial::SizeOverflow)?;
    let _backing = window
        .reserve_owned(requested)
        .map_err(|cause| Denial::Backing { requested, cause })?;
    let decoded = match DecodedPhysicalMutationBindingRecord::decode(payload, context) {
        Ok(decoded) => decoded,
        Err(PhysicalBindingCompactionRecordDecodeDenial::Persisted(
            PhysicalPersistedBindingDecodeDenial::Allocation { requested, cause },
        )) => {
            return Err(Denial::Backing {
                requested,
                cause: PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause },
            });
        }
        Err(PhysicalBindingCompactionRecordDecodeDenial::Persisted(
            PhysicalPersistedBindingDecodeDenial::AllocatorExceededReservation {
                requested,
                actual,
            },
        )) => return Err(Denial::AllocatorExceededReservation { requested, actual }),
        Err(_) => return Ok(None),
    };
    Ok(Some(checkpoint_evidence(decoded, selected_generation)))
}
