//! Admission of one exact durable-root physical frame.
use super::*;

impl DurablePhysicalRootManifest {
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
        let record_anchors = if frame.schema == 12 {
            RecordAnchorTable::decode(&frame.payload[HEAD_BOUND_ROOT_PAYLOAD_BYTES..])
                .ok_or(RootManifestDenial::InvalidRecordIdentity)?
        } else {
            RecordAnchorTable::EMPTY
        };
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
        .record_anchors(record_anchors)
        .admit()
        .map(|mut manifest| {
            manifest = manifest.with_root_schema(frame.schema);
            if frame.schema >= 10 {
                manifest.requires_maintenance_protocol = head_maintenance;
            }
            (manifest, format)
        })
        .ok_or(RootManifestDenial::InvalidPlacement)
    }
}
