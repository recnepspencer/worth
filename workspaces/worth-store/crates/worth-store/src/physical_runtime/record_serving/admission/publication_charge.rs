use worth_store_physical_format::DurablePhysicalRootManifest;

use super::super::planning::sealed_publication_overhead;

/// Metadata bytes each publication sealed, priced at the root that publication produced.
///
/// A later growth in routing height must not reprice older publications.
pub(super) fn retained_publication_overhead(roots: &[DurablePhysicalRootManifest]) -> u64 {
    roots.iter().filter(|root| root.generation() > 1).fold(0, |total, root| {
        total.saturating_add(sealed_publication_overhead(root))
    })
}
