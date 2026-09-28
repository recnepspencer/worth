use super::*;

mod cursor;
use cursor::*;
mod manifest;
use manifest::{read_manifest, write_manifest};
mod frame_coordinate;
use frame_coordinate::{read_subject_coordinate, write_subject_coordinate};

impl PersistedPhysicalRecoveryProjection {
    pub fn encode(&self) -> Vec<u8> {
        let mut target = Vec::new();
        field(&mut target, DOMAIN);
        target.extend_from_slice(&self.source_root_generation.to_le_bytes());
        field(&mut target, &self.root_state.encode());
        write_sequence(&mut target, &self.record_identities, |target, record| {
            write_record(target, *record)
        });
        match &self.payload {
            PersistedPhysicalRecoveryPayload::Frames(frames) => {
                target.push(0);
                write_sequence(&mut target, frames, write_frame);
            }
            PersistedPhysicalRecoveryPayload::SourceCopy(recipe) => {
                target.push(1);
                field(
                    &mut target,
                    &crate::PhysicalExtentCopyRecord::Intent(recipe.intent()).encode(),
                );
                target.extend_from_slice(&recipe.intent_lsn().to_le_bytes());
                target.extend_from_slice(&recipe.intent_digest());
            }
        }
        write_sequence(&mut target, &self.placements, write_placement);
        write_sequence(&mut target, &self.segment_updates, write_segment_update);
        write_sequence(&mut target, &self.manifests, write_manifest);
        target
    }

    pub fn decode(
        bytes: &[u8],
        limits: PhysicalRecoveryProjectionDecodeLimits,
        format: crate::PhysicalRecordFormatDeclaration,
    ) -> Result<Self, PhysicalRecoveryProjectionDenial> {
        Self::decode_payload(bytes, limits, Some(format))
    }

    /// Frame-only protocol owners cannot admit source-copy geometry without the
    /// bootstrap format. Reject that variant rather than inventing a default.
    pub fn decode_frames(
        bytes: &[u8],
        limits: PhysicalRecoveryProjectionDecodeLimits,
    ) -> Result<Self, PhysicalRecoveryProjectionDenial> {
        Self::decode_payload(bytes, limits, None)
    }

    fn decode_payload(
        bytes: &[u8],
        limits: PhysicalRecoveryProjectionDecodeLimits,
        format: Option<crate::PhysicalRecordFormatDeclaration>,
    ) -> Result<Self, PhysicalRecoveryProjectionDenial> {
        let mut cursor = Cursor::new(bytes);
        if cursor.field()? != DOMAIN {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        }
        let source_root_generation = cursor.u64()?;
        let root_state =
            PersistedPhysicalRecoveryRootState::decode(cursor.field()?, limits.inline_allocations)
                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        let record_identities = read_sequence(&mut cursor, limits.record_identities, |bytes| {
            let mut cursor = Cursor::new(bytes);
            let record = read_record(&mut cursor)?;
            cursor.end()?;
            Ok(record)
        })?;
        let payload = match cursor.byte()? {
            0 => PersistedPhysicalRecoveryPayload::Frames(
                read_sequence(&mut cursor, limits.frames, read_frame)?.into_boxed_slice(),
            ),
            1 => {
                let format = format.ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
                let crate::PhysicalExtentCopyRecord::Intent(intent) =
                    crate::PhysicalExtentCopyRecord::decode(cursor.field()?, format)
                        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?
                else {
                    return Err(PhysicalRecoveryProjectionDenial::Malformed);
                };
                let lsn = cursor.u64()?;
                let digest = cursor
                    .take(32)?
                    .try_into()
                    .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
                PersistedPhysicalRecoveryPayload::SourceCopy(
                    PersistedExtentCopyRecipe::new(intent, lsn, digest)
                        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
                )
            }
            _ => return Err(PhysicalRecoveryProjectionDenial::Malformed),
        };
        let mut remaining_entries = limits.total_entries;
        let placements = read_bounded_sequence(
            &mut cursor,
            limits.placements,
            &mut remaining_entries,
            read_placement,
        )?;
        let segment_updates = read_bounded_sequence(
            &mut cursor,
            limits.segment_updates,
            &mut remaining_entries,
            read_segment_update,
        )?;
        let manifests = read_bounded_sequence(
            &mut cursor,
            limits.manifests,
            &mut remaining_entries,
            read_manifest,
        )?;
        cursor.end()?;
        match payload {
            PersistedPhysicalRecoveryPayload::Frames(frames) => Self::new(
                source_root_generation,
                root_state,
                record_identities,
                frames.into_vec(),
                placements,
                segment_updates,
                manifests,
            ),
            PersistedPhysicalRecoveryPayload::SourceCopy(recipe) => {
                let expected = Self::from_source_copy(source_root_generation, root_state, recipe)
                    .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
                (record_identities.as_slice() == expected.record_identities()
                    && placements.as_slice() == expected.placements()
                    && segment_updates.is_empty()
                    && manifests.is_empty())
                .then_some(expected)
            }
        }
        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)
    }
}

fn write_sequence<T>(target: &mut Vec<u8>, values: &[T], write: fn(&mut Vec<u8>, &T)) {
    target.extend_from_slice(&(values.len() as u64).to_le_bytes());
    for value in values {
        let mut encoded = Vec::new();
        write(&mut encoded, value);
        field(target, &encoded);
    }
}

fn read_sequence<T>(
    cursor: &mut Cursor<'_>,
    maximum: u64,
    read: fn(&[u8]) -> Result<T, PhysicalRecoveryProjectionDenial>,
) -> Result<Vec<T>, PhysicalRecoveryProjectionDenial> {
    let count = cursor.u64()?;
    if count > maximum {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit);
    }
    let mut values = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(read(cursor.field()?)?);
    }
    Ok(values)
}

fn read_bounded_sequence<T>(
    cursor: &mut Cursor<'_>,
    maximum: u64,
    remaining_total: &mut u64,
    read: fn(&[u8]) -> Result<T, PhysicalRecoveryProjectionDenial>,
) -> Result<Vec<T>, PhysicalRecoveryProjectionDenial> {
    let count = cursor.u64()?;
    if count > maximum || count > *remaining_total {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit);
    }
    *remaining_total -= count;
    let mut values = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(read(cursor.field()?)?);
    }
    Ok(values)
}

fn write_frame(target: &mut Vec<u8>, frame: &PersistedPhysicalRecoveryFrame) {
    write_subject_coordinate(target, frame.subject, frame.coordinate);
    field(target, frame.bytes());
}

fn read_frame(
    bytes: &[u8],
) -> Result<PersistedPhysicalRecoveryFrame, PhysicalRecoveryProjectionDenial> {
    let mut cursor = Cursor::new(bytes);
    let (subject, coordinate) = read_subject_coordinate(&mut cursor)?;
    let payload = cursor.field()?;
    cursor.end()?;
    PersistedPhysicalRecoveryFrame::new(subject, coordinate, payload)
        .ok_or(PhysicalRecoveryProjectionDenial::InvalidFrame)
}

fn write_placement(target: &mut Vec<u8>, placement: &CurrentPhysicalRecordPlacement) {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(value) => {
            target.push(1);
            write_record(target, value.record());
            target.extend_from_slice(&value.segment().get().to_le_bytes());
            target.extend_from_slice(&value.segment_generation().to_le_bytes());
            target.extend_from_slice(&value.page().get().to_le_bytes());
            target.extend_from_slice(&value.page_generation().to_le_bytes());
            target.extend_from_slice(&value.slot().get().to_le_bytes());
            target.extend_from_slice(&value.slot_generation().to_le_bytes());
            target.extend_from_slice(&value.segment_page_capacity().to_le_bytes());
            target.extend_from_slice(&value.payload_bytes().to_le_bytes());
        }
        CurrentPhysicalRecordPlacement::Extent(value) => {
            target.push(2);
            write_record(target, value.record());
            target.extend_from_slice(&value.extent().get().to_le_bytes());
            target.extend_from_slice(&value.extent_generation().to_le_bytes());
            target.extend_from_slice(&value.payload_bytes().to_le_bytes());
            target.extend_from_slice(&value.arena_range().arena().get().to_le_bytes());
            target.extend_from_slice(&value.arena_range().offset().to_le_bytes());
            target.extend_from_slice(&value.arena_range().length().to_le_bytes());
        }
    }
}

fn read_placement(
    bytes: &[u8],
) -> Result<CurrentPhysicalRecordPlacement, PhysicalRecoveryProjectionDenial> {
    let mut cursor = Cursor::new(bytes);
    let kind = cursor.byte()?;
    let record = read_record(&mut cursor)?;
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let placement = match kind {
        1 => {
            let segment = PhysicalSegmentId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let segment_generation = generation(cursor.u64()?)?;
            let page = PhysicalPageId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let page_generation = generation(cursor.u64()?)?;
            let slot = PhysicalRecordSlot::from_raw(cursor.u16()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let slot_generation = generation(cursor.u64()?)?;
            let capacity = cursor.u32()?;
            let payload = cursor.u64()?;
            CurrentPhysicalRecordPlacement::Inline(
                DurableInlineRecordPlacement::new(
                    record,
                    authority
                        .segment_cell(segment)
                        .with_segment_generation(segment_generation),
                    authority
                        .page_cell(segment, page)
                        .with_page_generation(page_generation),
                    authority
                        .slot_cell(segment, page, slot)
                        .with_slot_generation(slot_generation),
                    capacity,
                    payload,
                )
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?,
            )
        }
        2 => {
            let extent = PhysicalExtentId::from_raw(cursor.u64()?)
                .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let extent_generation = generation(cursor.u64()?)?;
            let payload = cursor.u64()?;
            let arena = ExtentArenaId::new(cursor.u64()?)
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            let range = ExtentArenaRange::new(arena, cursor.u64()?, cursor.u64()?)
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?;
            CurrentPhysicalRecordPlacement::Extent(
                DurableExtentRecordPlacement::new(
                    record,
                    authority
                        .record_extent_cell(extent)
                        .with_extent_generation(extent_generation),
                    payload,
                    range,
                )
                .ok_or(PhysicalRecoveryProjectionDenial::InvalidPlacement)?,
            )
        }
        _ => return Err(PhysicalRecoveryProjectionDenial::InvalidPlacement),
    };
    cursor.end()?;
    Ok(placement)
}

fn write_segment_update(target: &mut Vec<u8>, update: &RecordSegmentPageManifestEntry) {
    target.extend_from_slice(&update.page_cell().segment_id().get().to_le_bytes());
    target.extend_from_slice(&update.page().get().to_le_bytes());
    target.extend_from_slice(&update.page_generation().to_le_bytes());
    target.extend_from_slice(&update.data_generation().to_le_bytes());
    target.extend_from_slice(&update.data_page_count().to_le_bytes());
    target.extend_from_slice(&update.frame_index().to_le_bytes());
}

fn read_segment_update(
    bytes: &[u8],
) -> Result<RecordSegmentPageManifestEntry, PhysicalRecoveryProjectionDenial> {
    let mut cursor = Cursor::new(bytes);
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(cursor.u64()?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidSegmentUpdate)?;
    let page = PhysicalPageId::from_raw(cursor.u64()?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::InvalidSegmentUpdate)?;
    let page_generation = generation(cursor.u64()?)?;
    let data_generation = generation(cursor.u64()?)?;
    let count = cursor.u32()?;
    let index = cursor.u32()?;
    cursor.end()?;
    RecordSegmentPageManifestEntry::new(
        authority
            .page_cell(segment, page)
            .with_page_generation(page_generation),
        authority
            .segment_cell(segment)
            .with_segment_generation(data_generation),
        count,
        index,
    )
    .ok_or(PhysicalRecoveryProjectionDenial::InvalidSegmentUpdate)
}
