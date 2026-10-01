//! The certified Batch checkpoint's selected root can be an addressed V3
//! source below the new pending publication, not the current selector root.

use super::*;
use crate::physical_runtime::recovery_construction::selected_rejoin::pending_wal_release::selection::Selection;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn observe_release_base(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &Selection,
    claim: &VerifiedSelectedCheckpointCustody,
    format: PhysicalRecordFormatDeclaration,
) -> Result<ObservedRootCheckpoint, Denial> {
    let selector =
        DurableRootSelector::decode(&selected.selector).map_err(|_| Denial::RootBinding)?;
    let (source, releases, count, bytes) = inspect_checkpoint(
        &selected.checkpoint_bytes,
        claim.checkpoint().source().identity(),
        claim.checkpoint().certificate_records(),
    )?;
    let observed = ObservedRootCheckpoint {
        selector,
        selector_bytes: selected.selector.clone(),
        root: selected.source_root.clone(),
        root_bytes: selected.source_bytes.clone(),
        root_sha256: Sha256::digest(&selected.source_bytes).into(),
        checkpoint_bytes: selected.checkpoint_bytes.clone(),
        checkpoint_source: source,
        release_certificates: releases,
        release_record_count: count,
        release_encoded_bytes: bytes,
    };
    if selector.format() != format || selector.store_identity() != discovery.store_identity() {
        return Err(Denial::RootBinding);
    }
    observed.matches_claim(discovery, claim)?;
    Ok(observed)
}
