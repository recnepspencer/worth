use crate::record_framing::{decode_durable_frame, FRAME_SCHEMA};
use crate::{
    DurableFrameDenial, DurableFrameKind, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration, PhysicalRootReference,
    PhysicalSegmentId, RootPublicationCell, SegmentGenerationCell,
};

use super::durable_root_routing::{
    decode_identity, decode_reference, encode_identity, encode_reference, ManifestBlockReference,
};
use super::durable_segment_routing::{
    decode_reference as decode_segment_reference, encode_reference as encode_segment_reference,
    SegmentManifestBlockReference,
};
use super::free_space_routing::{
    decode_reference as decode_free_space_reference,
    encode_reference as encode_free_space_reference, FreeSpaceBlockReference,
};
use super::release_custody_head::ReleaseCustodyHeadBlockReferenceV1;
use super::routing_tree_height::required_tree_level;
use super::{DerivedFamilyRootDirectoryBinding, IndexedThroughBlobPublication};

pub const CURRENT_ROOT_MANIFEST_PREFIX_BYTES: usize = 24;
pub const CURRENT_ROOT_MANIFEST_ENTRY_BYTES: usize = 88;
const LEGACY_ROOT_PAYLOAD_BYTES: usize = 336;
const DIRECTORY_BOUND_ROOT_PAYLOAD_BYTES: usize = 496;
const QUARANTINE_BOUND_ROOT_PAYLOAD_BYTES: usize = 528;
const TIER_ANCHORED_ROOT_PAYLOAD_BYTES: usize = 560;
const HEAD_BOUND_ROOT_PAYLOAD_BYTES: usize = 680;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurablePhysicalRootManifest {
    root: RootPublicationCell,
    tree_identity: u64,
    node_capacity: u16,
    record_count: u64,
    next_block: u64,
    next_segment_block: u64,
    free_space_checksum: u32,
    routing_root: Option<ManifestBlockReference>,
    segment_root: Option<SegmentManifestBlockReference>,
    free_space_root: Option<FreeSpaceBlockReference>,
    release_custody_head_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    next_release_custody_head_block: u64,
    latest_blob_publication: Option<IndexedThroughBlobPublication>,
    latest_blob_quarantine: Option<PersistedRecordIdentity>,
    tier_epoch_anchor: Option<[u8; 32]>,
    derived_family_directory: Option<DerivedFamilyRootDirectoryBinding>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
    requires_maintenance_protocol: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurablePhysicalRootManifestBuilder {
    generation: u64,
    tree_identity: u64,
    node_capacity: u16,
    free_space_checksum: u32,
    record_count: u64,
    next_block: u64,
    next_segment_block: u64,
    routing_root: Option<ManifestBlockReference>,
    segment_root: Option<SegmentManifestBlockReference>,
    free_space_root: Option<FreeSpaceBlockReference>,
    release_custody_head_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    next_release_custody_head_block: u64,
    latest_blob_publication: Option<IndexedThroughBlobPublication>,
    latest_blob_quarantine: Option<PersistedRecordIdentity>,
    tier_epoch_anchor: Option<[u8; 32]>,
    derived_family_directory: Option<DerivedFamilyRootDirectoryBinding>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
}

impl DurablePhysicalRootManifest {
    /// Maximum co-live payload and framed-output storage used by `encode`.
    /// Canonical admission may encode the current layout even for an older input.
    pub const fn maximum_encoding_scratch_bytes() -> usize {
        2 * HEAD_BOUND_ROOT_PAYLOAD_BYTES + crate::record_framing::DURABLE_FRAME_HEADER_BYTES
    }

    pub const fn builder(
        generation: u64,
        tree_identity: u64,
        node_capacity: u16,
        free_space_checksum: u32,
    ) -> DurablePhysicalRootManifestBuilder {
        DurablePhysicalRootManifestBuilder {
            generation,
            tree_identity,
            node_capacity,
            free_space_checksum,
            record_count: 0,
            next_block: 1,
            next_segment_block: 1,
            routing_root: None,
            segment_root: None,
            free_space_root: None,
            release_custody_head_root: None,
            next_release_custody_head_block: 1,
            latest_blob_publication: None,
            latest_blob_quarantine: None,
            tier_epoch_anchor: None,
            derived_family_directory: None,
            last_inline_record: None,
            last_inline_segment: None,
        }
    }

    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let length = self.encoded_frame_bytes();
        self.encode_in_reserved(format, Vec::with_capacity(length))
            .expect("reserved root manifest encoding")
    }

    pub fn encoded_frame_bytes(&self) -> usize {
        crate::record_framing::DURABLE_FRAME_HEADER_BYTES
            + if self.release_custody_head_root.is_some()
                || self.next_release_custody_head_block > 1
            {
                HEAD_BOUND_ROOT_PAYLOAD_BYTES
            } else if self.tier_epoch_anchor.is_some() {
                TIER_ANCHORED_ROOT_PAYLOAD_BYTES
            } else if self.latest_blob_quarantine.is_some() {
                QUARANTINE_BOUND_ROOT_PAYLOAD_BYTES
            } else if self.latest_blob_publication.is_some()
                || self.derived_family_directory.is_some()
            {
                DIRECTORY_BOUND_ROOT_PAYLOAD_BYTES
            } else {
                LEGACY_ROOT_PAYLOAD_BYTES
            }
    }

    pub fn encode_in_reserved(
        &self,
        format: PhysicalRecordFormatDeclaration,
        frame: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let payload_bytes =
            self.encoded_frame_bytes() - crate::record_framing::DURABLE_FRAME_HEADER_BYTES;
        let schema = if payload_bytes == HEAD_BOUND_ROOT_PAYLOAD_BYTES {
            10
        } else if self.tier_epoch_anchor.is_some() {
            9
        } else {
            FRAME_SCHEMA
                + u8::from(self.requires_maintenance_protocol)
                + if self.latest_blob_quarantine.is_some() {
                    4
                } else if self.latest_blob_publication.is_some()
                    || self.derived_family_directory.is_some()
                {
                    2
                } else {
                    0
                }
        };
        crate::record_framing::encode_durable_frame_in_reserved(
            DurableFrameKind::RootManifest,
            format,
            self.generation(),
            payload_bytes,
            schema,
            frame,
            |payload| {
                payload[..8].copy_from_slice(&self.generation().to_le_bytes());
                payload[8..16].copy_from_slice(&self.tree_identity.to_le_bytes());
                payload[16..18].copy_from_slice(&self.node_capacity.to_le_bytes());
                payload[24..32].copy_from_slice(&self.record_count.to_le_bytes());
                payload[32..40].copy_from_slice(&self.next_block.to_le_bytes());
                payload[152..156].copy_from_slice(&self.free_space_checksum.to_le_bytes());
                payload[224..232].copy_from_slice(&self.next_segment_block.to_le_bytes());
                if let Some(reference) = self.routing_root {
                    payload[40] = 1;
                    encode_reference(&mut payload[48..120], reference);
                }
                if let Some(record) = self.last_inline_record {
                    payload[120] = 1;
                    encode_identity(&mut payload[128..152], record);
                }
                if let Some(reference) = self.segment_root {
                    payload[160] = 1;
                    encode_segment_reference(&mut payload[168..224], reference);
                }
                if let Some(reference) = self.free_space_root {
                    payload[232] = 1;
                    encode_free_space_reference(&mut payload[240..312], reference);
                }
                if let Some(segment) = self.last_inline_segment {
                    payload[312] = 1;
                    payload[320..328].copy_from_slice(&segment.segment_id().get().to_le_bytes());
                    payload[328..336].copy_from_slice(&segment.generation().get().to_le_bytes());
                }
                if let Some(publication) = self.latest_blob_publication {
                    payload[401] = 1;
                    super::derived_family_directory::encode_publication(
                        &mut payload[336..400],
                        publication,
                    );
                }
                if let Some(binding) = self.derived_family_directory {
                    payload[400] = 1;
                    super::derived_family_directory::encode_root_binding(
                        &mut payload[408..496],
                        binding,
                    );
                }
                if let Some(quarantine) = self.latest_blob_quarantine {
                    payload[496] = 1;
                    encode_identity(&mut payload[504..528], quarantine);
                }
                if let Some(anchor) = self.tier_epoch_anchor {
                    payload[528..560].copy_from_slice(&anchor);
                }
                if payload.len() == HEAD_BOUND_ROOT_PAYLOAD_BYTES {
                    release_head_reference::encode(
                        payload,
                        self.release_custody_head_root,
                        self.next_release_custody_head_block,
                        self.requires_maintenance_protocol,
                    );
                }
            },
        )
    }

    pub fn decode(
        bytes: &[u8],
        max_entries: u16,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), RootManifestDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::RootManifest)
            .map_err(RootManifestDenial::Frame)?;
        let (latest_blob_publication, bound_directory, latest_blob_quarantine) =
            publication_fields::decode(frame.schema, frame.payload)?;
        if frame.payload[18..24] != [0; 6]
            || frame.payload[41..48] != [0; 7]
            || frame.payload[121..128] != [0; 7]
            || frame.payload[156..160] != [0; 4]
            || frame.payload[161..168] != [0; 7]
            || frame.payload[233..240] != [0; 7]
            || frame.payload[313..320] != [0; 7]
        {
            return Err(RootManifestDenial::MalformedPrefix);
        }
        let generation = u64::from_le_bytes(frame.payload[..8].try_into().unwrap());
        let tree_identity = u64::from_le_bytes(frame.payload[8..16].try_into().unwrap());
        let node_capacity = u16::from_le_bytes(frame.payload[16..18].try_into().unwrap());
        let record_count = u64::from_le_bytes(frame.payload[24..32].try_into().unwrap());
        let next_block = u64::from_le_bytes(frame.payload[32..40].try_into().unwrap());
        let free_space_checksum = u32::from_le_bytes(frame.payload[152..156].try_into().unwrap());
        let next_segment_block = u64::from_le_bytes(frame.payload[224..232].try_into().unwrap());
        let (tier_epoch_anchor, head_root, next_head_block, head_maintenance) =
            release_head_reference::decode(frame.schema, frame.payload)?;
        if generation == 0 || generation != frame.identity {
            return Err(RootManifestDenial::IdentityMismatch);
        }
        if node_capacity < 2 || node_capacity > max_entries {
            return Err(RootManifestDenial::EntryLimitExceeded);
        }
        let routing_root = match frame.payload[40] {
            0 => None,
            1 => Some(
                decode_reference(&frame.payload[48..120])
                    .ok_or(RootManifestDenial::InvalidPlacement)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let last_inline_record = match frame.payload[120] {
            0 => None,
            1 => Some(
                decode_identity(&frame.payload[128..152])
                    .ok_or(RootManifestDenial::InvalidRecordIdentity)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let segment_root = match frame.payload[160] {
            0 => None,
            1 => Some(
                decode_segment_reference(&frame.payload[168..224])
                    .ok_or(RootManifestDenial::InvalidPlacement)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let free_space_root = match frame.payload[232] {
            0 => None,
            1 => Some(
                decode_free_space_reference(&frame.payload[240..312])
                    .ok_or(RootManifestDenial::InvalidPlacement)?,
            ),
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        let last_inline_segment = match frame.payload[312] {
            0 => None,
            1 => {
                let segment = PhysicalSegmentId::from_raw(u64::from_le_bytes(
                    frame.payload[320..328].try_into().unwrap(),
                ))
                .map_err(|_| RootManifestDenial::InvalidPlacement)?;
                let generation = PhysicalGeneration::from_raw(u64::from_le_bytes(
                    frame.payload[328..336].try_into().unwrap(),
                ))
                .map_err(|_| RootManifestDenial::InvalidPlacement)?;
                Some(
                    PhysicalGenerationAuthority::for_canonical_physical_format()
                        .segment_cell(segment)
                        .with_segment_generation(generation),
                )
            }
            _ => return Err(RootManifestDenial::MalformedPrefix),
        };
        Self::builder(
            generation,
            tree_identity,
            node_capacity,
            free_space_checksum,
        )
        .record_count(record_count)
        .next_block(next_block)
        .next_segment_block(next_segment_block)
        .routing_root(routing_root)
        .segment_root(segment_root)
        .free_space_root(free_space_root)
        .release_custody_head_root(head_root)
        .next_release_custody_head_block(next_head_block)
        .latest_blob_publication(latest_blob_publication)
        .latest_blob_quarantine(latest_blob_quarantine)
        .tier_epoch_anchor(tier_epoch_anchor)
        .derived_family_directory(bound_directory)
        .last_inline_record(last_inline_record)
        .last_inline_segment(last_inline_segment)
        .admit()
        .map(|mut manifest| {
            manifest = manifest.with_root_schema(frame.schema);
            if frame.schema == 10 {
                manifest.requires_maintenance_protocol = head_maintenance;
            }
            (manifest, format)
        })
        .ok_or(RootManifestDenial::InvalidPlacement)
    }
}

#[path = "durable_root/accessors.rs"]
mod accessors;
#[path = "durable_root/builder.rs"]
mod builder;
#[path = "durable_root/capacity.rs"]
mod capacity;
#[path = "durable_root/publication_fields.rs"]
mod publication_fields;
#[path = "durable_root/release_head_reference.rs"]
mod release_head_reference;
pub use capacity::maximum_current_root_entries;

#[cfg(test)]
#[path = "durable_root/preallocated_encode_tests.rs"]
mod preallocated_encode_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootManifestDenial {
    Frame(DurableFrameDenial),
    MalformedPrefix,
    IdentityMismatch,
    EntryLimitExceeded,
    MalformedEntryLength,
    ReservedFieldNonZero,
    InvalidRecordIdentity,
    InvalidPlacement,
}

#[path = "maintenance.rs"]
mod maintenance;
