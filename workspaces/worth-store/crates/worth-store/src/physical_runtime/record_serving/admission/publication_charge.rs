use worth_store_physical_format::DurablePhysicalRootManifest;

use super::super::planning::sealed_publication_overhead;

/// Metadata bytes each publication sealed, oldest first, priced at the root
/// that publication produced.
///
/// A publication's charge is released when its WAL segment is reclaimed, so
/// reopen charges only the newest publications whose WAL frames remain.
pub(super) fn publication_overheads(roots: &[DurablePhysicalRootManifest]) -> Vec<u64> {
    roots
        .iter()
        .filter(|root| root.generation() > 1)
        .map(sealed_publication_overhead)
        .collect()
}
