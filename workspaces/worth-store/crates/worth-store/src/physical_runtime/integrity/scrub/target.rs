use super::PhysicalIntegrityScrubRequestDenial;
use worth_store_physical_format::{
    BlobRecordKind, PersistedRecordIdentity, PhysicalArtifactReadRange, PhysicalArtifactReadTarget,
};
use worth_store_physical_integrity::PhysicalArtifactScope;

/// The source of one bounded diagnostic window. A selected record has no raw
/// artifact address: its route and payload length come from a protected C.5 root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityScrubSource {
    Media(PhysicalArtifactReadRange),
    SelectedRecord(PersistedRecordIdentity),
}

/// Parent authority carried by the Store issuer, not a caller-supplied proof.
/// A valid inner checksum alone cannot bind a blob chunk to its selected edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum SelectedRecordScrubBasis {
    BTreeNode {
        family_code: u16,
        key_bytes: usize,
        leaf_value_bytes: usize,
    },
    BlobChunk {
        session: [u8; 16],
        ordinal: u64,
        chunk_size: u32,
        digest: [u8; 32],
        covered_bytes: u64,
    },
    BlobTreeNode {
        session: [u8; 16],
        level: u8,
        index: u64,
        digest: [u8; 32],
        covered_bytes: u64,
        root_digest: bool,
    },
    BlobGenerationPublication {
        object: [u8; 16],
        generation: u64,
    },
}

/// An expected integrity family and bounded acquisition source, never read
/// authority. Only Store owners can issue the selected-record form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityScrubTarget {
    scope: PhysicalArtifactScope,
    source: PhysicalIntegrityScrubSource,
    issued_under: Option<crate::physical_runtime::PhysicalProtectedRootObservation>,
    basis: Option<SelectedRecordScrubBasis>,
}

impl PhysicalIntegrityScrubTarget {
    pub fn new(
        target: PhysicalArtifactReadTarget,
        scope: PhysicalArtifactScope,
    ) -> Result<Self, PhysicalIntegrityScrubRequestDenial> {
        use PhysicalArtifactReadTarget as Target;
        let matches = match target {
            Target::Record(artifact) => {
                super::super::resident_admission::artifact_matches_scope(artifact, scope)
            }
            Target::Wal(identity) => scope.wal_segment_identity() == Some(identity),
            Target::Checkpoint(identity) => scope.checkpoint_identity() == Some(identity),
            Target::PhysicalWork(identity) => {
                scope.physical_work_obligation_identity() == Some(identity)
            }
        };
        if !matches {
            return Err(PhysicalIntegrityScrubRequestDenial::TargetScopeMismatch);
        }
        let length = u32::try_from(scope.byte_range().length())
            .map_err(|_| PhysicalIntegrityScrubRequestDenial::WindowBoundExceeded)?;
        let range = PhysicalArtifactReadRange::new(target, scope.byte_range().offset(), length)
            .ok_or(PhysicalIntegrityScrubRequestDenial::TargetScopeMismatch)?;
        Ok(Self {
            scope,
            source: PhysicalIntegrityScrubSource::Media(range),
            issued_under: None,
            basis: None,
        })
    }

    /// The scope length is an acquisition ceiling, not asserted on-media
    /// length. The selected C.5 route supplies the actual length at inspection.
    pub(in crate::physical_runtime) fn selected_record(
        scope: PhysicalArtifactScope,
        issued_under: crate::physical_runtime::PhysicalProtectedRootObservation,
        basis: SelectedRecordScrubBasis,
    ) -> Result<Self, PhysicalIntegrityScrubRequestDenial> {
        let record = scope
            .btree_node_identity()
            .map(|(record, _)| record)
            .or_else(|| scope.blob_record_identity().map(|(record, _)| record))
            .ok_or(PhysicalIntegrityScrubRequestDenial::TargetScopeMismatch)?;
        if scope.byte_range().offset() != 0 {
            return Err(PhysicalIntegrityScrubRequestDenial::TargetScopeMismatch);
        }
        u32::try_from(scope.byte_range().length())
            .map_err(|_| PhysicalIntegrityScrubRequestDenial::WindowBoundExceeded)?;
        let matches_basis = match (
            scope.btree_node_identity(),
            scope.blob_record_identity(),
            basis,
        ) {
            (
                Some((_, selected_family)),
                None,
                SelectedRecordScrubBasis::BTreeNode {
                    family_code,
                    key_bytes,
                    leaf_value_bytes,
                },
            ) => selected_family == family_code && key_bytes > 0 && leaf_value_bytes > 0,
            (
                None,
                Some((_, BlobRecordKind::Chunk)),
                SelectedRecordScrubBasis::BlobChunk { .. },
            )
            | (
                None,
                Some((_, BlobRecordKind::TreeNode)),
                SelectedRecordScrubBasis::BlobTreeNode { .. },
            )
            | (
                None,
                Some((_, BlobRecordKind::GenerationPublished)),
                SelectedRecordScrubBasis::BlobGenerationPublication { .. },
            ) => true,
            _ => false,
        };
        if !matches_basis {
            return Err(PhysicalIntegrityScrubRequestDenial::TargetScopeMismatch);
        }
        Ok(Self {
            scope,
            source: PhysicalIntegrityScrubSource::SelectedRecord(record),
            issued_under: Some(issued_under),
            basis: Some(basis),
        })
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn source(self) -> PhysicalIntegrityScrubSource {
        self.source
    }

    pub const fn media_range(self) -> Option<PhysicalArtifactReadRange> {
        match self.source {
            PhysicalIntegrityScrubSource::Media(range) => Some(range),
            PhysicalIntegrityScrubSource::SelectedRecord(_) => None,
        }
    }

    pub fn declared_bytes(self) -> u32 {
        u32::try_from(self.scope.byte_range().length())
            .expect("admitted scrub target has a u32 window bound")
    }

    /// Descriptive root selected by the Store issuer; not read authority.
    pub const fn issued_under(
        self,
    ) -> Option<crate::physical_runtime::PhysicalProtectedRootObservation> {
        self.issued_under
    }

    pub(super) const fn basis(self) -> Option<SelectedRecordScrubBasis> {
        self.basis
    }

    pub(super) fn overlaps(self, other: Self) -> bool {
        match (self.source, other.source) {
            (
                PhysicalIntegrityScrubSource::Media(left),
                PhysicalIntegrityScrubSource::Media(right),
            ) => left.overlaps(right),
            (
                PhysicalIntegrityScrubSource::SelectedRecord(left),
                PhysicalIntegrityScrubSource::SelectedRecord(right),
            ) => left == right,
            _ => false,
        }
    }

    pub(super) fn selected_scope(self, actual_payload_bytes: u64) -> Option<PhysicalArtifactScope> {
        if actual_payload_bytes > self.scope.byte_range().length() {
            return None;
        }
        let range =
            worth_store_physical_integrity::PhysicalByteRange::new(0, actual_payload_bytes).ok()?;
        let store = self.scope.store_identity();
        if let Some((record, family_code)) = self.scope.btree_node_identity() {
            return Some(PhysicalArtifactScope::btree_node(
                store,
                record,
                family_code,
                range,
            ));
        }
        let (record, kind) = self.scope.blob_record_identity()?;
        match kind {
            BlobRecordKind::Chunk => Some(PhysicalArtifactScope::blob_chunk_frame(
                store, record, range,
            )),
            BlobRecordKind::TreeNode => {
                Some(PhysicalArtifactScope::blob_tree_node(store, record, range))
            }
            BlobRecordKind::GenerationPublished => Some(
                PhysicalArtifactScope::blob_generation_publication(store, record, range),
            ),
            _ => None,
        }
    }
}
