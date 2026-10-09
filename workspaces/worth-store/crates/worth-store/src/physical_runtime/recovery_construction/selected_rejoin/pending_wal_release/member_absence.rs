//! Store's own proof that no replayed C.9 member leaves a source root. A
//! retirement edge has no member; Store rereads every sampled member at or
//! above the checkpoint cutoff instead of trusting the plan's edge kind.

use worth_store_physical_format::{
    decode_canonical_redo_v3, decode_canonical_redo_v3_with_storage,
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_wal::WalLsnRange;

use super::super::{
    resident::{
        decode_storage::{decode_denial, RejoinDecodeStorage, RejoinDecoded},
        StoreRejoinResidentLedger,
    },
    SelectedMediaRejoinDenial as Denial,
};
use super::ordinary_member::decode_limits;
use crate::physical_runtime::{
    PhysicalRecoveryReadAllocation, StoreRecoveryBindingFreshnessSample,
};

/// Whether any sampled member at or above `cutoff` leaves `source_generation`,
/// decoding each member within `maximum_decode_scratch_bytes`.
pub(super) fn member_leaves(
    sample: &StoreRecoveryBindingFreshnessSample,
    cutoff: u64,
    source_generation: u64,
    maximum_decode_scratch_bytes: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Result<bool, Denial> {
    any_member_leaves(
        members(sample),
        cutoff,
        source_generation,
        |bytes, range, bound, limits| {
            if bound.checked_mul(4).ok_or(Denial::BoundExceeded)? > maximum_decode_scratch_bytes {
                return Err(Denial::BoundExceeded);
            }
            let (_, projection) = decode_canonical_redo_v3(
                bytes,
                range.start().get(),
                range.end_exclusive().get(),
                bound,
                None,
                limits,
                format,
            )
            .map_err(|_| Denial::WalFate)?;
            Ok(projection.source_root_generation())
        },
    )
}

/// The funded form: each decode is admitted in the native Recovery pool and
/// released before the next member is read.
pub(super) fn member_leaves_with_storage(
    sample: &StoreRecoveryBindingFreshnessSample,
    cutoff: u64,
    source_generation: u64,
    format: PhysicalRecordFormatDeclaration,
    window: &PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<bool, Denial> {
    any_member_leaves(
        members(sample),
        cutoff,
        source_generation,
        |bytes, range, bound, limits| {
            let mut storage = RejoinDecodeStorage::new(window, resident);
            let decoded = decode_canonical_redo_v3_with_storage(
                bytes,
                range.start().get(),
                range.end_exclusive().get(),
                bound,
                limits,
                format,
                &mut storage,
            )
            .map_err(decode_denial)?;
            RejoinDecoded::new(decoded, storage.into_charge())
                .with_data(resident, |(_, projection), _| {
                    Ok(projection.source_root_generation())
                })
        },
    )
}

fn members(
    sample: &StoreRecoveryBindingFreshnessSample,
) -> impl Iterator<Item = (WalLsnRange, &[u8])> {
    sample
        .wal_members()
        .iter()
        .map(|member| (member.lsn_range(), member.canonical_redo()))
}

fn any_member_leaves<'a>(
    members: impl Iterator<Item = (WalLsnRange, &'a [u8])>,
    cutoff: u64,
    source_generation: u64,
    mut decode_source: impl FnMut(
        &[u8],
        WalLsnRange,
        u64,
        PhysicalRecoveryProjectionDecodeLimits,
    ) -> Result<u64, Denial>,
) -> Result<bool, Denial> {
    for (range, redo) in members {
        if range.start().get() < cutoff {
            continue;
        }
        let bound = redo.len() as u64;
        if decode_source(redo, range, bound, decode_limits(bound))? == source_generation {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
#[path = "member_absence_tests.rs"]
mod tests;
