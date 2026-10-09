//! Typed root-only cleanup frames do not belong to ordinary mutation groups.

use worth_store_physical_format::BlobManifestResidueCleanup;
use worth_store_wal::WalLsnRange;

use super::{PhysicalWalBindingReopenCutoff, PhysicalWalOpenFailure};

pub(super) fn observe(
    payload: &[u8],
    range: WalLsnRange,
    store: [u8; 16],
    cutoff: PhysicalWalBindingReopenCutoff,
    retained_spans: &mut Vec<(u64, u64)>,
) -> Result<(), PhysicalWalOpenFailure> {
    let cleanup = BlobManifestResidueCleanup::decode(payload)
        .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
    if cleanup.store() != store {
        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
    }
    match cutoff.lsn() {
        Some(cutoff_lsn) if range.end_exclusive() <= cutoff_lsn => {}
        Some(cutoff_lsn) if range.start() < cutoff_lsn => {
            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
        }
        _ => retained_spans.push((range.start().get(), range.end_exclusive().get())),
    }
    Ok(())
}
