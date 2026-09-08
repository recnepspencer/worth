use sha2::{Digest, Sha256};
use worth_store_physical_format::RecordFrameCoordinate;

use super::PhysicalWorkIntent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalWorkConcurrencyScope {
    digest: [u8; 32],
    security: worth_store_security::StoreSecurityScopeIdentity,
    coordinates: Box<[RecordFrameCoordinate]>,
    inspection: Option<worth_store_physical_format::PhysicalArtifactReadRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkConcurrencyRelation {
    DisjointArtifacts,
    SameArtifactDisjointRanges,
    Overlapping,
}

impl PhysicalWorkConcurrencyScope {
    pub(in crate::physical_runtime::work) fn derive(intent: &PhysicalWorkIntent) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"worth-store.physical-concurrency-scope.v1");
        digest.update(intent.identity().store().bytes());
        digest.update(intent.security().stable_fingerprint());
        digest.update(intent.scope().stable_digest());
        Self {
            digest: digest.finalize().into(),
            security: intent.security(),
            coordinates: intent.scope().coordinates().into(),
            inspection: intent.scope().inspection_target(),
        }
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub const fn security(&self) -> worth_store_security::StoreSecurityScopeIdentity {
        self.security
    }

    pub fn coordinates(&self) -> &[RecordFrameCoordinate] {
        &self.coordinates
    }

    pub fn relation(&self, other: &Self) -> PhysicalWorkConcurrencyRelation {
        if self.inspection.is_some() || other.inspection.is_some() {
            return inspection_relation(self, other);
        }
        let mut same_artifact = false;
        for left in &self.coordinates {
            for right in &other.coordinates {
                if left.artifact() != right.artifact() {
                    continue;
                }
                same_artifact = true;
                let left_end = left.offset().saturating_add(u64::from(left.length()));
                let right_end = right.offset().saturating_add(u64::from(right.length()));
                if left.offset() < right_end && right.offset() < left_end {
                    return PhysicalWorkConcurrencyRelation::Overlapping;
                }
            }
        }
        if same_artifact {
            PhysicalWorkConcurrencyRelation::SameArtifactDisjointRanges
        } else {
            PhysicalWorkConcurrencyRelation::DisjointArtifacts
        }
    }
}

fn inspection_relation(
    left: &PhysicalWorkConcurrencyScope,
    right: &PhysicalWorkConcurrencyScope,
) -> PhysicalWorkConcurrencyRelation {
    use worth_store_physical_format::PhysicalArtifactReadTarget;
    use PhysicalWorkConcurrencyRelation as Relation;
    if let (Some(left), Some(right)) = (left.inspection, right.inspection) {
        return if !left.target().same_location(right.target()) {
            Relation::DisjointArtifacts
        } else if left.overlaps(right) {
            Relation::Overlapping
        } else {
            Relation::SameArtifactDisjointRanges
        };
    }
    let (inspection, coordinates) = match left.inspection {
        Some(range) => (range, &right.coordinates),
        None => (
            right.inspection.expect("one inspection scope"),
            &left.coordinates,
        ),
    };
    // Non-range owners may replace a namespace or retire an artifact. Their
    // empty coordinate projection is not evidence of disjointness.
    if coordinates.is_empty() {
        return Relation::Overlapping;
    }
    let mut same_artifact = false;
    for coordinate in coordinates {
        if inspection.target() != PhysicalArtifactReadTarget::Record(coordinate.artifact()) {
            continue;
        }
        same_artifact = true;
        let end = coordinate
            .offset()
            .saturating_add(coordinate.length() as u64);
        if inspection.offset() < end
            && coordinate.offset() < inspection.offset() + inspection.length() as u64
        {
            return Relation::Overlapping;
        }
    }
    if same_artifact {
        Relation::SameArtifactDisjointRanges
    } else {
        Relation::DisjointArtifacts
    }
}
