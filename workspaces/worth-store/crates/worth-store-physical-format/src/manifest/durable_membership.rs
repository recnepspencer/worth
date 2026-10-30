use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    DurableFrameDenial, DurableFrameKind, PageGenerationCell, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordFormatDeclaration,
    PhysicalSegmentId, SegmentGenerationCell,
};

pub const fn maximum_segment_manifest_pages(format: PhysicalRecordFormatDeclaration) -> u32 {
    let available = format.page_size().bytes() as usize
        - crate::record_framing::DURABLE_FRAME_HEADER_BYTES
        - 24;
    let pages = available / 40;
    if pages > u32::MAX as usize {
        u32::MAX
    } else {
        pages as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordSegmentPageManifestEntry {
    page: PageGenerationCell,
    data_segment: SegmentGenerationCell,
    data_page_count: u32,
    frame_index: u32,
}

impl RecordSegmentPageManifestEntry {
    pub const fn new(
        page: PageGenerationCell,
        data_segment: SegmentGenerationCell,
        data_page_count: u32,
        frame_index: u32,
    ) -> Option<Self> {
        if page.segment_id().get() != data_segment.segment_id().get()
            || data_page_count == 0
            || frame_index >= data_page_count
        {
            return None;
        }
        Some(Self {
            page,
            data_segment,
            data_page_count,
            frame_index,
        })
    }
    pub const fn page(self) -> PhysicalPageId {
        self.page.page_id()
    }
    pub const fn page_cell(self) -> PageGenerationCell {
        self.page
    }
    pub const fn page_generation(self) -> u64 {
        self.page.generation().get()
    }
    pub const fn data_segment_cell(self) -> SegmentGenerationCell {
        self.data_segment
    }
    pub const fn data_generation(self) -> u64 {
        self.data_segment.generation().get()
    }
    pub const fn data_page_count(self) -> u32 {
        self.data_page_count
    }
    pub const fn frame_index(self) -> u32 {
        self.frame_index
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableSegmentManifest {
    segment: SegmentGenerationCell,
    page_capacity: u32,
    pages: Vec<RecordSegmentPageManifestEntry>,
}

impl DurableSegmentManifest {
    pub fn new(
        format: PhysicalRecordFormatDeclaration,
        segment: SegmentGenerationCell,
        page_capacity: u32,
        pages: Vec<RecordSegmentPageManifestEntry>,
    ) -> Option<Self> {
        if page_capacity == 0
            || page_capacity > maximum_segment_manifest_pages(format)
            || pages.is_empty()
            || pages.len() > page_capacity as usize
        {
            return None;
        }
        let mut identities = BTreeSet::new();
        let mut data_coordinates = BTreeSet::new();
        let mut data_shapes = BTreeMap::new();
        for page in &pages {
            if page.page_cell().segment_id() != segment.segment_id()
                || page.data_segment_cell().segment_id() != segment.segment_id()
                || page.data_generation() > segment.generation().get()
                || page.data_page_count() > page_capacity
                || !identities.insert(page.page())
                || !data_coordinates.insert((page.data_generation(), page.frame_index()))
                || data_shapes
                    .insert(page.data_generation(), page.data_page_count())
                    .is_some_and(|old| old != page.data_page_count())
            {
                return None;
            }
        }
        Some(Self {
            segment,
            page_capacity,
            pages,
        })
    }
    pub const fn segment(&self) -> PhysicalSegmentId {
        self.segment.segment_id()
    }
    pub const fn segment_cell(&self) -> SegmentGenerationCell {
        self.segment
    }
    pub const fn generation(&self) -> u64 {
        self.segment.generation().get()
    }
    pub const fn page_capacity(&self) -> u32 {
        self.page_capacity
    }
    pub fn pages(&self) -> &[RecordSegmentPageManifestEntry] {
        &self.pages
    }
    pub fn locate(&self, page: PhysicalPageId) -> Option<RecordSegmentPageManifestEntry> {
        self.pages
            .iter()
            .find(|entry| entry.page() == page)
            .copied()
    }
    pub fn encode(&self, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
        let mut payload = vec![0_u8; 24 + self.pages.len() * 40];
        payload[..8].copy_from_slice(&self.segment().get().to_le_bytes());
        payload[8..12].copy_from_slice(&self.page_capacity.to_le_bytes());
        payload[12..16].copy_from_slice(&(self.pages.len() as u32).to_le_bytes());
        for (index, page) in self.pages.iter().enumerate() {
            let base = 24 + index * 40;
            payload[base..base + 8].copy_from_slice(&page.page().get().to_le_bytes());
            payload[base + 8..base + 16].copy_from_slice(&page.page_generation().to_le_bytes());
            payload[base + 16..base + 24].copy_from_slice(&page.data_generation().to_le_bytes());
            payload[base + 24..base + 28].copy_from_slice(&page.data_page_count().to_le_bytes());
            payload[base + 28..base + 32].copy_from_slice(&page.frame_index().to_le_bytes());
        }
        encode_durable_frame(
            DurableFrameKind::SegmentManifest,
            format,
            self.generation(),
            &payload,
        )
    }
    pub fn decode(
        bytes: &[u8],
        maximum_pages: u32,
    ) -> Result<(Self, PhysicalRecordFormatDeclaration), MembershipManifestDenial> {
        let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::SegmentManifest)
            .map_err(MembershipManifestDenial::Frame)?;
        if frame.payload.len() < 24 || frame.payload[16..24] != [0; 8] {
            return Err(MembershipManifestDenial::Malformed);
        }
        let segment =
            PhysicalSegmentId::from_raw(u64::from_le_bytes(frame.payload[..8].try_into().unwrap()))
                .map_err(|_| MembershipManifestDenial::Malformed)?;
        let capacity = u32::from_le_bytes(frame.payload[8..12].try_into().unwrap());
        let count = u32::from_le_bytes(frame.payload[12..16].try_into().unwrap());
        if frame.identity == 0
            || capacity == 0
            || capacity > maximum_pages
            || count == 0
            || count > capacity
            || count > maximum_pages
            || frame.payload.len() != 24 + count as usize * 40
        {
            return Err(MembershipManifestDenial::Limit);
        }
        let mut pages = Vec::with_capacity(count as usize);
        let generation = PhysicalGeneration::from_raw(frame.identity)
            .map_err(|_| MembershipManifestDenial::Malformed)?;
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment_cell = authority
            .segment_cell(segment)
            .with_segment_generation(generation);
        for chunk in frame.payload[24..].as_chunks::<40>().0.iter() {
            if chunk[32..40] != [0; 8] {
                return Err(MembershipManifestDenial::Reserved);
            }
            let page = PhysicalPageId::from_raw(u64::from_le_bytes(chunk[..8].try_into().unwrap()))
                .map_err(|_| MembershipManifestDenial::Malformed)?;
            let page_generation =
                PhysicalGeneration::from_raw(u64::from_le_bytes(chunk[8..16].try_into().unwrap()))
                    .map_err(|_| MembershipManifestDenial::Malformed)?;
            let data_generation =
                PhysicalGeneration::from_raw(u64::from_le_bytes(chunk[16..24].try_into().unwrap()))
                    .map_err(|_| MembershipManifestDenial::Malformed)?;
            pages.push(
                RecordSegmentPageManifestEntry::new(
                    authority
                        .page_cell(segment, page)
                        .with_page_generation(page_generation),
                    authority
                        .segment_cell(segment)
                        .with_segment_generation(data_generation),
                    u32::from_le_bytes(chunk[24..28].try_into().unwrap()),
                    u32::from_le_bytes(chunk[28..32].try_into().unwrap()),
                )
                .ok_or(MembershipManifestDenial::Malformed)?,
            );
        }
        Self::new(format, segment_cell, capacity, pages)
            .map(|value| (value, format))
            .ok_or(MembershipManifestDenial::Malformed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipManifestDenial {
    Frame(DurableFrameDenial),
    Malformed,
    Limit,
    Reserved,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format() -> PhysicalRecordFormatDeclaration {
        PhysicalRecordFormatDeclaration::builder().admit().unwrap()
    }

    fn generation(raw: u64) -> PhysicalGeneration {
        PhysicalGeneration::from_raw(raw).unwrap()
    }

    #[test]
    fn segment_membership_rejects_impossible_capacity_and_data_artifact_shapes() {
        let format = format();
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment_id = PhysicalSegmentId::from_raw(1).unwrap();
        let segment = authority
            .segment_cell(segment_id)
            .with_segment_generation(generation(2));
        let data_segment = authority
            .segment_cell(segment_id)
            .with_segment_generation(generation(1));
        let page = |id, count| {
            RecordSegmentPageManifestEntry::new(
                authority
                    .page_cell(segment_id, PhysicalPageId::from_raw(id).unwrap())
                    .with_page_generation(generation(1)),
                data_segment,
                count,
                id as u32 - 1,
            )
            .unwrap()
        };

        assert!(
            DurableSegmentManifest::new(format, segment, 2, vec![page(1, 2), page(2, 2)]).is_some()
        );
        assert!(DurableSegmentManifest::new(
            format,
            segment,
            maximum_segment_manifest_pages(format) + 1,
            vec![page(1, 1)],
        )
        .is_none());
        assert!(
            DurableSegmentManifest::new(format, segment, 2, vec![page(1, 1), page(2, 2)]).is_none()
        );
        assert!(DurableSegmentManifest::new(format, segment, 2, vec![page(1, 3)]).is_none());
    }
}
