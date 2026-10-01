//! Parsed C.11 selected-record claims, distinct from their graph verdicts.

use worth_foundational::PhysicalArtifactFamily as Family;

use super::super::OfflineArtifactFamily;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BlobFact {
    Abandoned {
        store: [u8; 16],
        frame_digest: [u8; 32],
        session: [u8; 16],
        declaration_record: [u8; 24],
        declaration_digest: [u8; 32],
        expiry_checkpoint: Option<u64>,
    },
    Declaration {
        store: [u8; 16],
        session: [u8; 16],
        frame_digest: [u8; 32],
        object: [u8; 16],
        scope: [u8; 32],
        chunk_size: u32,
        total: u64,
        max_checkpoint_sequence: u64,
    },
    Chunk {
        store: [u8; 16],
        session: [u8; 16],
        ordinal: u64,
        chunk_size: u32,
        length: u64,
        digest: [u8; 32],
    },
    ReuseClaim {
        store: [u8; 16],
        session: [u8; 16],
        ordinal: u64,
        scope: [u8; 32],
        chunk_size: u32,
        length: u64,
        digest: [u8; 32],
        chunk_record: [u8; 24],
        source_publication: [u8; 24],
        source_ordinal: u64,
        source_witness: Option<ReuseSourceWitness>,
    },
    DedupeQuarantine {
        store: [u8; 16],
        scope: [u8; 32],
        digest: [u8; 32],
        chunk_size: u32,
        source_publication: [u8; 24],
        source_ordinal: u64,
        source_chunk: [u8; 24],
        destination_session: [u8; 16],
        destination_ordinal: u64,
        conflicting_chunk: [u8; 24],
    },
    Node {
        store: [u8; 16],
        session: [u8; 16],
        kind: u8,
        level: u8,
        index: u64,
        covered: u64,
        digest: [u8; 32],
        frame_digest: [u8; 32],
        entries: Vec<BlobEdge>,
    },
    Publication {
        store: [u8; 16],
        frame_digest: [u8; 32],
        session: [u8; 16],
        object: [u8; 16],
        generation: u64,
        root: [u8; 24],
        root_digest: [u8; 32],
        total: u64,
        logical_digest: [u8; 32],
        chunk_size: u32,
        scope: [u8; 32],
    },
    Frontier {
        store: [u8; 16],
        session: [u8; 16],
        declaration_record: [u8; 24],
        declaration_digest: [u8; 32],
        next_chunk_ordinal: u64,
        durable_bytes: u64,
        last_chunk_record: [u8; 24],
        last_chunk_digest: [u8; 32],
    },
    DropSetManifest {
        store: [u8; 16],
        frame_digest: [u8; 32],
        attempt: [u8; 16],
        session: [u8; 16],
        declaration_record: [u8; 24],
        declaration_digest: [u8; 32],
        abandoned_record: [u8; 24],
        abandoned_digest: [u8; 32],
        basis_digest: [u8; 32],
        dropped: Vec<[u8; 24]>,
        never_reserved_slot_generation: Option<u64>,
    },
    ReleasedDropSetManifest {
        store: [u8; 16],
        frame_digest: [u8; 32],
        attempt: [u8; 16],
        object: [u8; 16],
        session: [u8; 16],
        generation: u64,
        root: [u8; 24],
        root_digest: [u8; 32],
        publication_record: [u8; 24],
        publication_digest: [u8; 32],
        issuer_evidence_digest: [u8; 32],
        basis_digest: [u8; 32],
        dropped: Vec<[u8; 24]>,
        never_reserved_slot_generation: u64,
    },
    OriginalDropReserved {
        store: [u8; 16],
        frame_digest: [u8; 32],
        attempt: [u8; 16],
        manifest_record: [u8; 24],
        manifest_digest: [u8; 32],
        basis_digest: [u8; 32],
        manifest_selected_generation: u64,
        reserved_selected_generation: u64,
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
        lease_issuance_generation: u64,
        lease_expiry_generation: u64,
    },
    ReclaimDescriptor {
        store: [u8; 16],
        attempt: [u8; 16],
        basis_digest: [u8; 32],
        manifest_record: [u8; 24],
        manifest_digest: [u8; 32],
        manifest_count: u16,
        source_root: u64,
        candidate_root: u64,
    },
    ReleasedReclaimDescriptor {
        store: [u8; 16],
        frame_digest: [u8; 32],
        attempt: [u8; 16],
        basis_digest: [u8; 32],
        manifest_record: [u8; 24],
        manifest_digest: [u8; 32],
        manifest_count: u16,
        source_root: u64,
        candidate_root: u64,
        predecessor: Option<([u8; 24], [u8; 32])>,
        cumulative_dropped: u64,
        terminal: bool,
        custody: Option<ReleasedDropCustodyFact>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReleasedDropCustodyFact {
    pub(crate) digest: [u8; 32],
    pub(crate) source_root_sha256: [u8; 32],
    pub(crate) source_free_space_sha256: [u8; 32],
    pub(crate) selected_routes_sha256: [u8; 32],
    pub(crate) closure_sha256: [u8; 32],
    pub(crate) external_edges_sha256: [u8; 32],
    pub(crate) postorder_sha256: [u8; 32],
    pub(crate) idempotency: [u8; 32],
    pub(crate) fingerprint: [u8; 32],
    pub(crate) lease_issuance: u64,
    pub(crate) lease_expiry: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReuseSourceWitness {
    pub(crate) frame_digest: [u8; 32],
    pub(crate) store: [u8; 16],
    pub(crate) session: [u8; 16],
    pub(crate) object: [u8; 16],
    pub(crate) generation: u64,
    pub(crate) root: [u8; 24],
    pub(crate) root_digest: [u8; 32],
    pub(crate) total: u64,
    pub(crate) chunk_size: u32,
    pub(crate) scope: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlobEdge {
    pub(crate) digest: [u8; 32],
    pub(crate) record: [u8; 24],
    pub(crate) covered: u64,
}

impl BlobFact {
    pub(crate) const fn route_kind_matches(&self, code: u8) -> bool {
        match self {
            Self::Declaration { .. } => code == 1,
            Self::Chunk { .. } => code == 2,
            Self::Node { .. } => code == 3,
            Self::Publication { .. } => code == 4,
            Self::Frontier { .. } => code == 5,
            Self::Abandoned { .. } => code == 6,
            Self::DropSetManifest { .. } => matches!(code, 7 | 9),
            Self::ReclaimDescriptor { .. } => code == 8,
            Self::OriginalDropReserved { .. } => code == 10,
            Self::ReuseClaim { source_witness, .. } => {
                if source_witness.is_some() {
                    code == 15
                } else {
                    code == 11
                }
            }
            Self::DedupeQuarantine { .. } => code == 12,
            Self::ReleasedDropSetManifest { .. } => code == 13,
            Self::ReleasedReclaimDescriptor { custody, .. } => {
                if custody.is_some() {
                    code == 16
                } else {
                    code == 14
                }
            }
        }
    }
    pub(crate) const fn family(&self) -> OfflineArtifactFamily {
        match self {
            Self::Declaration { .. } | Self::Abandoned { .. } | Self::ReuseClaim { .. } => {
                OfflineArtifactFamily::Declared(Family::BlobResumeSession)
            }
            Self::DedupeQuarantine { .. } => OfflineArtifactFamily::DedupeQuarantine,
            Self::Chunk { .. } => OfflineArtifactFamily::Declared(Family::BlobChunkFrame),
            Self::Node { .. } => OfflineArtifactFamily::Declared(Family::BlobTreeNode),
            Self::Publication { .. } => {
                OfflineArtifactFamily::Declared(Family::BlobGenerationPublication)
            }
            Self::Frontier { .. } => OfflineArtifactFamily::Declared(Family::BlobResumeSession),
            Self::DropSetManifest { .. } | Self::ReleasedDropSetManifest { .. } => {
                OfflineArtifactFamily::Declared(Family::BlobDropSetManifest)
            }
            Self::OriginalDropReserved { .. } => OfflineArtifactFamily::OriginalDropReservation,
            Self::ReclaimDescriptor { .. } | Self::ReleasedReclaimDescriptor { .. } => {
                OfflineArtifactFamily::Declared(Family::BlobReclaimDescriptor)
            }
        }
    }

    pub(crate) const fn store(&self) -> [u8; 16] {
        match self {
            Self::Declaration { store, .. }
            | Self::Abandoned { store, .. }
            | Self::Chunk { store, .. }
            | Self::ReuseClaim { store, .. }
            | Self::DedupeQuarantine { store, .. }
            | Self::Node { store, .. }
            | Self::Publication { store, .. }
            | Self::Frontier { store, .. } => *store,
            Self::DropSetManifest { store, .. }
            | Self::ReleasedDropSetManifest { store, .. }
            | Self::OriginalDropReserved { store, .. }
            | Self::ReclaimDescriptor { store, .. }
            | Self::ReleasedReclaimDescriptor { store, .. } => *store,
        }
    }

    pub(crate) const fn session(&self) -> Option<[u8; 16]> {
        match self {
            Self::Declaration { session, .. }
            | Self::Abandoned { session, .. }
            | Self::Chunk { session, .. }
            | Self::ReuseClaim { session, .. }
            | Self::Node { session, .. }
            | Self::Publication { session, .. }
            | Self::Frontier { session, .. }
            | Self::DropSetManifest { session, .. } => Some(*session),
            Self::DedupeQuarantine { .. }
            | Self::ReleasedDropSetManifest { .. }
            | Self::OriginalDropReserved { .. }
            | Self::ReclaimDescriptor { .. }
            | Self::ReleasedReclaimDescriptor { .. } => None,
        }
    }
}
