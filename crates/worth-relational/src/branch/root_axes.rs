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
/// Unlike the incremental publication path, this deliberately reconstructs
/// every partition digest from authoritative content. Its cost belongs to the
/// recovery or inspection lane, where a carried descriptor must not be able to
/// relabel different bytes as the same root.
pub(super) fn storage_root_from_authoritative_regions(
    regions: &RelationalPersistentRegionSet,
    symbols: &crate::symbols::data::StringInterner,
) -> Result<([u8; 32], [u8; 32]), super::RelationalBranchRootCaptureDenial> {
    for region in regions.values() {
        let content_digest = region
            .partition
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
    digest.update(b"worth.relational.branch-storage-content.v3\0");
    digest.update((region_count as u64).to_be_bytes());
    digest.update(commitment);
    digest.finalize().into()
}
