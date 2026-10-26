use super::RelationalPersistentRegionSet;

pub(super) fn storage_root_for(regions: &RelationalPersistentRegionSet) -> ([u8; 32], [u8; 32]) {
    let commitment = regions.commitment();
    (
        storage_root_from_commitment(regions.len(), commitment),
        commitment,
    )
}

/// Recovery/inspection court for the immutable bytes represented by a root.
///
/// Resident payloads are checked against their leaves. Cold leaves are bound
/// by the root commitment here and their payloads are verified on demand;
/// checking a descriptor never initiates region I/O.
pub(super) fn storage_root_from_authoritative_regions(
    regions: &RelationalPersistentRegionSet,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<([u8; 32], [u8; 32]), super::RelationalBranchRootCaptureDenial> {
    for region in regions.values() {
        let Some(lease) = region
            .partition
            .cell()
            .lease_resident_full()
            .expect("content inspection retains resident data")
        else {
            continue;
        };
        let content_digest = lease
            .full_view()
            .partition()
            .authoritative_content_digest(symbols)
            .map_err(|error| match error {
                crate::storage::overlay::PartitionContentDigestError::UnresolvedContentSymbol(
                    symbol,
                ) => super::RelationalBranchRootCaptureDenial::UnresolvedContentSymbol(symbol),
            })?;
        if content_digest != region.content_digest {
            return Err(
                super::RelationalBranchRootCaptureDenial::StorageContentMismatch {
                    descriptor_root: region.content_digest,
                    reconstructed_root: content_digest,
                },
            );
        }
    }
    Ok(storage_root_for(regions))
}

fn storage_root_from_commitment(region_count: usize, commitment: [u8; 32]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-storage-content.v4\0");
    digest.update((region_count as u64).to_be_bytes());
    digest.update(commitment);
    digest.finalize().into()
}
