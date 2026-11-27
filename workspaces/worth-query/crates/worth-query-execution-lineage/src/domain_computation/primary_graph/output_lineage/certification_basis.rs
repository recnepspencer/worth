//! The basis a re-certification compares at (public #429).
//!
//! A `FamilySelection` fact re-resolves its family from the lineage heads at
//! the certifier's product coordinate, so every comparison that can reach one
//! carries that coordinate with its snapshot. The type replaces the bare
//! snapshot on those paths: a caller with no product observation cannot build
//! one, so no re-certification path can lose the coordinate.

use worth_relational::facade::snapshots::SnapshotHandle;
use worth_runtime_world::facade::{ProductBranchIncarnation, ProductBranchReferenceGeneration};

use super::ProductCoordinate;

/// The snapshot a comparison reads and the product coordinate that selected it.
#[derive(Clone, Copy, Debug)]
pub struct CertificationBasis<'a> {
    pub snapshot: &'a SnapshotHandle,
    coordinate: ProductCoordinate,
}

impl<'a> CertificationBasis<'a> {
    /// The basis of a comparison at `snapshot` for the product occurrence and
    /// reference generation its product observation, lease or receipt holds.
    pub fn of(
        occurrence: ProductBranchIncarnation,
        generation: ProductBranchReferenceGeneration,
        snapshot: &'a SnapshotHandle,
    ) -> Self {
        Self {
            snapshot,
            coordinate: ProductCoordinate { occurrence, generation: generation.get() },
        }
    }

    /// The basis of a lineage owner that already holds its coordinate.
    pub(crate) fn at(coordinate: ProductCoordinate, snapshot: &'a SnapshotHandle) -> Self {
        Self { snapshot, coordinate }
    }

    pub fn coordinate(&self) -> ProductCoordinate {
        self.coordinate
    }
}
