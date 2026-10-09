use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use crate::{
    DurableFrameKind, PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration, RecordExtentGenerationCell,
    DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};

use super::MembershipManifestDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurableExtentManifest {
    record: PersistedRecordIdentity,
    extent: RecordExtentGenerationCell,
    logical_bytes: u64,
    maximum_frame_bytes: u32,
    chunk_count: u32,
    alignment: u64,
}

impl DurableExtentManifest {
    pub fn new(
        format: PhysicalRecordFormatDeclaration,
        record: PersistedRecordIdentity,
        extent: RecordExtentGenerationCell,
        logical_bytes: u64,
        maximum_frame_bytes: u32,
        chunk_count: u32,
        alignment: u64,
    ) -> Option<Self> {
        let overhead = DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES;
        if !alignment.is_power_of_two()
            || logical_bytes == 0
            || maximum_frame_bytes as usize <= overhead
            || maximum_frame_bytes != format.page_size().bytes()
        {
            return None;
        }
        let payload_capacity = maximum_frame_bytes as usize - overhead;
        let expected_chunks = u32::try_from(logical_bytes.div_ceil(payload_capacity as u64)).ok();
        if expected_chunks != Some(chunk_count) {
            return None;
        }
        Some(Self {
            record,
            extent,
            logical_bytes,
            maximum_frame_bytes,
            chunk_count,
            alignment,
        })
    }

    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn extent(self) -> PhysicalExtentId {
        self.extent.extent_id()
    }
    pub const fn extent_cell(self) -> RecordExtentGenerationCell {
        self.extent
    }
    pub const fn generation(self) -> u64 {
        self.extent.generation().get()
    }
    pub const fn logical_bytes(self) -> u64 {
        self.logical_bytes
    }
    pub const fn maximum_frame_bytes(self) -> u32 {
        self.maximum_frame_bytes
    }
    pub const fn chunk_payload_capacity(self) -> u32 {
        self.maximum_frame_bytes
            - (DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u32
    }
    pub const fn chunk_count(self) -> u32 {
        self.chunk_count
    }
    pub const fn alignment(self) -> u64 {
        self.alignment
    }

    pub fn encode(self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let mut payload = [0_u8; 56];
        payload[..16].copy_from_slice(&self.record.allocation_epoch());
        payload[16..24].copy_from_slice(&self.record.ordinal().to_le_bytes());
        payload[24..32].copy_from_slice(&self.extent().get().to_le_bytes());
        payload[32..40].copy_from_slice(&self.logical_bytes.to_le_bytes());
        payload[40..44].copy_from_slice(&self.maximum_frame_bytes.to_le_bytes());
        payload[44..48].copy_from_slice(&self.chunk_count.to_le_bytes());
        payload[48..56].copy_from_slice(&self.alignment.to_le_bytes());
        encode_durable_frame(
            DurableFrameKind::ExtentManifest,
            format,
            self.generation(),
            &payload,
        )
    }

    pub fn decode(
        bytes: &[u8],
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), MembershipManifestDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::ExtentManifest)
            .map_err(MembershipManifestDenial::Frame)?;
        if frame.payload.len() != 56 {
            return Err(MembershipManifestDenial::Malformed);
        }
        let record = PersistedRecordIdentity::new(
            frame.payload[..16].try_into().unwrap(),
            u64::from_le_bytes(frame.payload[16..24].try_into().unwrap()),
        )
        .ok_or(MembershipManifestDenial::Malformed)?;
        let extent = PhysicalExtentId::from_raw(u64::from_le_bytes(
            frame.payload[24..32].try_into().unwrap(),
        ))
        .map_err(|_| MembershipManifestDenial::Malformed)?;
        let generation = PhysicalGeneration::from_raw(frame.identity)
            .map_err(|_| MembershipManifestDenial::Malformed)?;
        Self::new(
            format,
            record,
            PhysicalGenerationAuthority::for_canonical_physical_format()
                .record_extent_cell(extent)
                .with_extent_generation(generation),
            u64::from_le_bytes(frame.payload[32..40].try_into().unwrap()),
            u32::from_le_bytes(frame.payload[40..44].try_into().unwrap()),
            u32::from_le_bytes(frame.payload[44..48].try_into().unwrap()),
            u64::from_le_bytes(frame.payload[48..56].try_into().unwrap()),
        )
        .map(|value| (value, format))
        .ok_or(MembershipManifestDenial::Malformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_requires_canonical_frame_width_and_exact_chunk_count() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(1).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
        let record = PersistedRecordIdentity::new([7; 16], 1).unwrap();
        let payload = format.page_size().bytes()
            - (DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u32;
        let logical = u64::from(payload) + 1;

        assert!(DurableExtentManifest::new(
            format,
            record,
            extent,
            logical,
            format.page_size().bytes(),
            2,
            4096,
        )
        .is_some());
        assert!(DurableExtentManifest::new(
            format,
            record,
            extent,
            logical,
            format.page_size().bytes(),
            1,
            4096,
        )
        .is_none());
        assert!(DurableExtentManifest::new(
            format,
            record,
            extent,
            logical,
            format.page_size().bytes() / 2,
            3,
            4096,
        )
        .is_none());
        assert!(DurableExtentManifest::new(
            format,
            record,
            extent,
            logical,
            format.page_size().bytes() + 1,
            2,
            4096,
        )
        .is_none());
    }
}
