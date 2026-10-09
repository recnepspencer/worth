use crate::PersistedRecordIdentity;

use super::super::envelope::{
    nonzero_16, nonzero_32, visit_canonical_frame, BLOB_RECORD_HEADER_BYTES,
};
use super::super::{BlobRecordDenial, BlobRecordKind};
use super::basis::read_record;
use super::{BlobReclaimSourceKind, MAXIMUM_DROP_SET_RECORDS};

pub(super) const PAYLOAD_BYTES: usize = 205;

/// Authenticated prior selected drop in a released-generation batch chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleasedDropPredecessorV1 {
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
}

impl ReleasedDropPredecessorV1 {
    pub fn new(
        descriptor_record: PersistedRecordIdentity,
        descriptor_frame_sha256: [u8; 32],
    ) -> Result<Self, BlobRecordDenial> {
        Ok(Self {
            descriptor_record,
            descriptor_frame_sha256: nonzero_32(descriptor_frame_sha256)?,
        })
    }

    pub const fn descriptor_record(self) -> PersistedRecordIdentity {
        self.descriptor_record
    }
    pub const fn descriptor_frame_sha256(self) -> [u8; 32] {
        self.descriptor_frame_sha256
    }
}

/// The source tag binds the selected manifest to the durable drop intent.
/// Recovery still verifies selected source records before applying the drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReclaimDescriptorV2 {
    store: [u8; 16],
    reclaim_attempt: [u8; 16],
    source_kind: BlobReclaimSourceKind,
    source_basis_digest: [u8; 32],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    manifest_count: u16,
    source_root_generation: u64,
    candidate_root_generation: u64,
    predecessor: Option<ReleasedDropPredecessorV1>,
    cumulative_dropped: u64,
    terminal: bool,
}

impl BlobReclaimDescriptorV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        reclaim_attempt: [u8; 16],
        source_kind: BlobReclaimSourceKind,
        source_basis_digest: [u8; 32],
        manifest_record: PersistedRecordIdentity,
        manifest_frame_sha256: [u8; 32],
        manifest_count: u16,
        source_root_generation: u64,
        candidate_root_generation: u64,
        predecessor: Option<ReleasedDropPredecessorV1>,
        cumulative_dropped: u64,
        terminal: bool,
    ) -> Result<Self, BlobRecordDenial> {
        if manifest_count == 0
            || usize::from(manifest_count) > MAXIMUM_DROP_SET_RECORDS
            || source_root_generation == 0
            || source_root_generation.checked_add(1) != Some(candidate_root_generation)
            || cumulative_dropped < u64::from(manifest_count)
            || (predecessor.is_none() && cumulative_dropped != u64::from(manifest_count))
            || (predecessor.is_some() && cumulative_dropped == u64::from(manifest_count))
            || (predecessor.is_some() && source_kind != BlobReclaimSourceKind::ReleasedGeneration)
        {
            return Err(BlobRecordDenial::InvalidReclaimDescriptor);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            reclaim_attempt: nonzero_16(reclaim_attempt)?,
            source_kind,
            source_basis_digest: nonzero_32(source_basis_digest)?,
            manifest_record,
            manifest_frame_sha256: nonzero_32(manifest_frame_sha256)?,
            manifest_count,
            source_root_generation,
            candidate_root_generation,
            predecessor,
            cumulative_dropped,
            terminal,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut frame = Vec::with_capacity(BLOB_RECORD_HEADER_BYTES + PAYLOAD_BYTES);
        visit_canonical_frame(
            BlobRecordKind::ReclaimDescriptorV2,
            PAYLOAD_BYTES,
            |emit| self.visit_payload(emit),
            &mut |part| frame.extend_from_slice(part),
        )
        .expect("fixed reclaim descriptor fits control-frame ceiling");
        frame
    }

    pub(super) fn visit_payload(self, emit: &mut dyn FnMut(&[u8])) {
        emit(&self.store);
        emit(&self.reclaim_attempt);
        emit(&[self.source_kind as u8]);
        emit(&self.source_basis_digest);
        emit(&self.manifest_record.allocation_epoch());
        emit(&self.manifest_record.ordinal().to_le_bytes());
        emit(&self.manifest_frame_sha256);
        emit(&self.manifest_count.to_le_bytes());
        emit(&self.source_root_generation.to_le_bytes());
        emit(&self.candidate_root_generation.to_le_bytes());
        if let Some(predecessor) = self.predecessor {
            emit(&[1]);
            emit(&predecessor.descriptor_record().allocation_epoch());
            emit(&predecessor.descriptor_record().ordinal().to_le_bytes());
            emit(&predecessor.descriptor_frame_sha256());
        } else {
            emit(&[0; 57]);
        }
        emit(&self.cumulative_dropped.to_le_bytes());
        emit(&[u8::from(self.terminal)]);
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::ReclaimDescriptorV2 {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload)
    }

    pub(in crate::blob_record) fn decode_payload(payload: &[u8]) -> Result<Self, BlobRecordDenial> {
        if payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::LengthMismatch);
        }
        let predecessor = match payload[139] {
            0 if payload[140..196] == [0; 56] => None,
            1 => Some(ReleasedDropPredecessorV1::new(
                read_record(&payload[140..164])?,
                payload[164..196]
                    .try_into()
                    .expect("fixed predecessor digest"),
            )?),
            _ => return Err(BlobRecordDenial::InvalidReclaimDescriptor),
        };
        let terminal = match payload[204] {
            0 => false,
            1 => true,
            _ => return Err(BlobRecordDenial::InvalidReclaimDescriptor),
        };
        Self::new(
            payload[0..16].try_into().expect("fixed store"),
            payload[16..32].try_into().expect("fixed attempt"),
            BlobReclaimSourceKind::decode(payload[32])?,
            payload[33..65].try_into().expect("fixed source digest"),
            read_record(&payload[65..89])?,
            payload[89..121].try_into().expect("fixed manifest digest"),
            u16::from_le_bytes(payload[121..123].try_into().expect("fixed count")),
            u64::from_le_bytes(payload[123..131].try_into().expect("fixed source root")),
            u64::from_le_bytes(payload[131..139].try_into().expect("fixed candidate root")),
            predecessor,
            u64::from_le_bytes(
                payload[196..204]
                    .try_into()
                    .expect("fixed cumulative count"),
            ),
            terminal,
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn reclaim_attempt(self) -> [u8; 16] {
        self.reclaim_attempt
    }
    pub const fn source_kind(self) -> BlobReclaimSourceKind {
        self.source_kind
    }
    pub const fn source_basis_digest(self) -> [u8; 32] {
        self.source_basis_digest
    }
    pub const fn manifest_record(self) -> PersistedRecordIdentity {
        self.manifest_record
    }
    pub const fn manifest_frame_sha256(self) -> [u8; 32] {
        self.manifest_frame_sha256
    }
    pub const fn manifest_count(self) -> u16 {
        self.manifest_count
    }
    pub const fn source_root_generation(self) -> u64 {
        self.source_root_generation
    }
    pub const fn candidate_root_generation(self) -> u64 {
        self.candidate_root_generation
    }
    pub const fn predecessor(self) -> Option<ReleasedDropPredecessorV1> {
        self.predecessor
    }
    pub const fn cumulative_dropped(self) -> u64 {
        self.cumulative_dropped
    }
    pub const fn terminal(self) -> bool {
        self.terminal
    }
}
