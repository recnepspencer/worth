//! The basis a re-certification compares at (public #429).
//!
//! A `FamilySelection` fact re-resolves its family from the lineage heads at
//! the certifier's product coordinate, so every comparison that can reach one
//! carries that coordinate with its snapshot. The type replaces the bare
//! snapshot on those paths: a caller with no product observation cannot build
//! one, so no re-certification path can lose the coordinate.

use std::sync::Arc;

use worth_relational::facade::snapshots::SnapshotHandle;
use worth_runtime_world::facade::{ProductBranchIncarnation, ProductBranchReferenceGeneration};

use super::{ProductCoordinate, RecordedSettlementIdentity};

/// The snapshot a comparison reads and the product coordinate that selected it.
#[derive(Clone, Copy, Debug)]
pub struct CertificationBasis<'a> {
    pub snapshot: &'a SnapshotHandle,
    coordinate: ProductCoordinate,
    /// The transported chain rows a historical readmission verified at this
    /// basis, which stand in for their own family heads (plan §3.5, S1 STOP 3).
    readmitted: &'a [Arc<RecordedSettlementIdentity>],
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
            readmitted: &[],
        }
    }

    /// The basis of a lineage owner that already holds its coordinate.
    pub(crate) fn at(coordinate: ProductCoordinate, snapshot: &'a SnapshotHandle) -> Self {
        Self { snapshot, coordinate, readmitted: &[] }
    }

    /// This basis for a historical readmission's own facts. `rows` are the
    /// consumer's upstream chain, re-identified at its transported coordinate
    /// after that readmission verified the chain at this basis. A family head
    /// that is exactly one of them is current here without a comparison; no
    /// other row is.
    pub fn with_readmitted(self, rows: &'a [Arc<RecordedSettlementIdentity>]) -> Self {
        Self { readmitted: rows, ..self }
    }

    /// Whether `identity` is a row this basis's readmission verified.
    pub(crate) fn readmitted(&self, identity: &RecordedSettlementIdentity) -> bool {
        self.readmitted.iter().any(|row| **row == *identity)
    }

    pub fn coordinate(&self) -> ProductCoordinate {
        self.coordinate
    }
}
