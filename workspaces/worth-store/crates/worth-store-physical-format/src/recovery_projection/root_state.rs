use super::{
    decode_storage, PhysicalRecoveryDecodeFailure, PhysicalRecoveryDecodeStorage,
    PhysicalRecoveryProjectionDenial,
};
use crate::{
    PersistedRecordIdentity, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalSegmentId,
    SegmentGenerationCell,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedPhysicalRecoveryRootState {
    root_publication_allocation_bytes: u64,
    manifest_capacity_transition: u8,
    successor_manifest_capacity: u16,
    inline_allocations: Box<[PersistedInlineSegmentAllocation]>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedInlineSegmentAllocation {
    segment: SegmentGenerationCell,
    page_capacity: u32,
    used_pages: u32,
}

impl PersistedPhysicalRecoveryRootState {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.inline_allocations.len())
            .ok()?
            .checked_mul(
                u64::try_from(std::mem::size_of::<PersistedInlineSegmentAllocation>()).ok()?,
            )
    }

    pub fn new(
        root_publication_allocation_bytes: u64,
        manifest_capacity_transition: u8,
        successor_manifest_capacity: u16,
        inline_allocations: Vec<PersistedInlineSegmentAllocation>,
        last_inline_record: Option<PersistedRecordIdentity>,
        last_inline_segment: Option<SegmentGenerationCell>,
    ) -> Option<Self> {
        (root_publication_allocation_bytes != 0
            && matches!(manifest_capacity_transition, 1 | 2)
            && successor_manifest_capacity >= 2
            && inline_allocations
                .windows(2)
                .all(|pair| allocation_key(pair[0]) < allocation_key(pair[1]))
            && last_inline_record.is_some() == last_inline_segment.is_some())
        .then_some(Self {
            root_publication_allocation_bytes,
            manifest_capacity_transition,
            successor_manifest_capacity,
            inline_allocations: inline_allocations.into_boxed_slice(),
            last_inline_record,
            last_inline_segment,
        })
    }

    pub const fn root_publication_allocation_bytes(&self) -> u64 {
        self.root_publication_allocation_bytes
    }
    pub const fn manifest_capacity_transition(&self) -> u8 {
        self.manifest_capacity_transition
    }
    pub const fn successor_manifest_capacity(&self) -> u16 {
        self.successor_manifest_capacity
    }
    pub fn inline_allocations(&self) -> &[PersistedInlineSegmentAllocation] {
        &self.inline_allocations
    }
    pub const fn last_inline_record(&self) -> Option<PersistedRecordIdentity> {
        self.last_inline_record
    }
    pub const fn last_inline_segment(&self) -> Option<SegmentGenerationCell> {
        self.last_inline_segment
    }

    pub(super) fn encode(&self) -> Vec<u8> {
        let mut target = Vec::new();
        target.extend_from_slice(&self.root_publication_allocation_bytes.to_le_bytes());
        target.push(self.manifest_capacity_transition);
        target.extend_from_slice(&self.successor_manifest_capacity.to_le_bytes());
        target.extend_from_slice(&(self.inline_allocations.len() as u64).to_le_bytes());
        for allocation in &self.inline_allocations {
            target.extend_from_slice(&allocation.segment.segment_id().get().to_le_bytes());
            target.extend_from_slice(&allocation.segment.generation().get().to_le_bytes());
            target.extend_from_slice(&allocation.page_capacity.to_le_bytes());
            target.extend_from_slice(&allocation.used_pages.to_le_bytes());
        }
        write_optional_record(&mut target, self.last_inline_record);
        match self.last_inline_segment {
            Some(segment) => {
                target.push(1);
                target.extend_from_slice(&segment.segment_id().get().to_le_bytes());
                target.extend_from_slice(&segment.generation().get().to_le_bytes());
            }
            None => target.push(0),
        }
        target
    }

    pub(super) fn decode_with_storage<S: PhysicalRecoveryDecodeStorage>(
        bytes: &[u8],
        maximum_allocations: u64,
        storage: &mut S,
    ) -> Result<Self, PhysicalRecoveryDecodeFailure<S::Denial>> {
        let malformed = || PhysicalRecoveryProjectionDenial::Malformed;
        let mut cursor = Cursor::new(bytes);
        let root_bytes = cursor.u64().ok_or_else(malformed)?;
        let transition = cursor.byte().ok_or_else(malformed)?;
        let successor_capacity = cursor.u16().ok_or_else(malformed)?;
        let count = cursor.u64().ok_or_else(malformed)?;
        if count > maximum_allocations {
            return Err(PhysicalRecoveryProjectionDenial::EntryLimit.into());
        }
        let minimum = usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(24))
            .and_then(|bytes| bytes.checked_add(2));
        if minimum.is_none_or(|minimum| minimum > cursor.remaining.len()) {
            return Err(malformed().into());
        }
        let mut allocations = decode_storage::reserve_vec(
            usize::try_from(count).map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
            storage,
        )?;
        for _ in 0..count {
            let segment = segment_cell(
                cursor.u64().ok_or_else(malformed)?,
                cursor.u64().ok_or_else(malformed)?,
            )
            .ok_or_else(malformed)?;
            allocations.push(
                PersistedInlineSegmentAllocation::new(
                    segment,
                    cursor.u32().ok_or_else(malformed)?,
                    cursor.u32().ok_or_else(malformed)?,
                )
                .ok_or_else(malformed)?,
            );
        }
        let last_record = read_optional_record(&mut cursor).ok_or_else(malformed)?;
        let last_segment = match cursor.byte().ok_or_else(malformed)? {
            0 => None,
            1 => Some(
                segment_cell(
                    cursor.u64().ok_or_else(malformed)?,
                    cursor.u64().ok_or_else(malformed)?,
                )
                .ok_or_else(malformed)?,
            ),
            _ => return Err(malformed().into()),
        };
        cursor.end().ok_or_else(malformed)?;
        Self::new(
            root_bytes,
            transition,
            successor_capacity,
            allocations,
            last_record,
            last_segment,
        )
        .ok_or_else(|| malformed().into())
    }
}

impl PersistedInlineSegmentAllocation {
    pub fn new(
        segment: SegmentGenerationCell,
        page_capacity: u32,
        used_pages: u32,
    ) -> Option<Self> {
        if page_capacity == 0 || used_pages == 0 || used_pages > page_capacity {
            return None;
        }
        Some(Self {
            segment,
            page_capacity,
            used_pages,
        })
    }
    pub const fn segment(self) -> SegmentGenerationCell {
        self.segment
    }
    pub const fn page_capacity(self) -> u32 {
        self.page_capacity
    }
    pub const fn used_pages(self) -> u32 {
        self.used_pages
    }
}

fn allocation_key(allocation: PersistedInlineSegmentAllocation) -> (u64, u64) {
    (
        allocation.segment.segment_id().get(),
        allocation.segment.generation().get(),
    )
}

#[cfg(test)]
mod decode_admission_tests {
    use super::*;
    #[test]
    fn truncated_inline_roster_denies_before_allocation() {
        struct Admission(usize);
        impl PhysicalRecoveryDecodeStorage for Admission {
            type Denial = ();
            fn admit_allocation(&mut self, _: u64) -> Result<(), ()> {
                self.0 += 1;
                Ok(())
            }
        }
        let root = PersistedPhysicalRecoveryRootState::new(1, 1, 2, vec![], None, None).unwrap();
        let mut bytes = root.encode();
        bytes[11..19].copy_from_slice(&1_u64.to_le_bytes());
        let mut storage = Admission(0);
        let result =
            PersistedPhysicalRecoveryRootState::decode_with_storage(&bytes, 1, &mut storage);
        assert!(matches!(
            result,
            Err(PhysicalRecoveryDecodeFailure::Projection(
                PhysicalRecoveryProjectionDenial::Malformed
            ))
        ));
        assert_eq!(storage.0, 0);
    }
}

fn segment_cell(segment: u64, generation: u64) -> Option<SegmentGenerationCell> {
    Some(
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .segment_cell(PhysicalSegmentId::from_raw(segment).ok()?)
            .with_segment_generation(PhysicalGeneration::from_raw(generation).ok()?),
    )
}

fn write_optional_record(target: &mut Vec<u8>, record: Option<PersistedRecordIdentity>) {
    match record {
        Some(record) => {
            target.push(1);
            target.extend_from_slice(&record.allocation_epoch());
            target.extend_from_slice(&record.ordinal().to_le_bytes());
        }
        None => target.push(0),
    }
}

fn read_optional_record(cursor: &mut Cursor<'_>) -> Option<Option<PersistedRecordIdentity>> {
    match cursor.byte()? {
        0 => Some(None),
        1 => Some(Some(PersistedRecordIdentity::new(
            cursor.take(16)?.try_into().ok()?,
            cursor.u64()?,
        )?)),
        _ => None,
    }
}

struct Cursor<'a> {
    remaining: &'a [u8],
}
impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }
    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let (head, tail) = self.remaining.split_at_checked(len)?;
        self.remaining = tail;
        Some(head)
    }
    fn byte(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    fn end(self) -> Option<()> {
        self.remaining.is_empty().then_some(())
    }
}
