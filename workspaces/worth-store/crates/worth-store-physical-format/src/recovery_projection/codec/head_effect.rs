use super::*;
use crate::{
    ReleaseCustodyHeadBlockReferenceV1 as HeadRef, ReleaseCustodyHeadEntryV1 as HeadEntry,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
    ReleasedGenerationReclaimBasisV1,
};

const REF_BYTES: usize = HeadRef::ENCODED_BYTES;
const ENTRY_BYTES: usize = HeadEntry::ENCODED_BYTES;

pub(super) fn encode_head_effect(effect: &PersistedReleaseCustodyHeadEffectV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&effect.tree_identity().to_le_bytes());
    field(&mut bytes, &effect.source_basis().encode());
    write_optional_ref(&mut bytes, effect.source_root());
    bytes.extend_from_slice(&effect.source_next_block().to_le_bytes());
    let ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior,
        next,
    } = effect.mutation()
    else {
        unreachable!("only upsert effects can be constructed in V14");
    };
    bytes.push(1);
    write_optional_entry(&mut bytes, expected_prior);
    write_entry(&mut bytes, next);
    write_ref(&mut bytes, effect.result_root());
    bytes.extend_from_slice(&effect.result_next_block().to_le_bytes());
    write_sequence(&mut bytes, effect.source_path(), |bytes, node| {
        write_ref(bytes, node.reference());
        field(bytes, node.frame());
    });
    write_sequence(&mut bytes, effect.node_writes(), |bytes, write| {
        write_ref(bytes, write.reference());
        field(bytes, write.frame());
    });
    bytes
}

pub(super) fn decode_head_effect(
    bytes: &[u8],
    source_root_generation: u64,
    remaining_entries: &mut u64,
    format: crate::PhysicalRecordFormatDeclaration,
) -> Result<PersistedReleaseCustodyHeadEffectV1, PhysicalRecoveryProjectionDenial> {
    let mut cursor = Cursor::new(bytes);
    let tree_identity = cursor.u64()?;
    let source_basis = ReleasedGenerationReclaimBasisV1::decode(cursor.field()?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
    let source_root = read_optional_ref(&mut cursor)?;
    let source_next_block = cursor.u64()?;
    if cursor.byte()? != 1 {
        return Err(PhysicalRecoveryProjectionDenial::Malformed);
    }
    let expected_prior = read_optional_entry(&mut cursor)?;
    let next = read_entry(&mut cursor)?;
    let mutation = ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior,
        next,
    };
    let result_root = read_ref(&mut cursor)?;
    let result_next_block = cursor.u64()?;
    let path = read_bounded_sequence(
        &mut cursor,
        *remaining_entries,
        remaining_entries,
        |bytes| {
            let mut cursor = Cursor::new(bytes);
            let reference = read_ref(&mut cursor)?;
            let frame = cursor.field()?;
            if frame.len() > format.page_size().bytes() as usize {
                return Err(PhysicalRecoveryProjectionDenial::EntryLimit);
            }
            cursor.end()?;
            Ok(ReleaseCustodyHeadPathNodeV1::new(reference, frame.to_vec()))
        },
    )?;
    let writes = read_bounded_sequence(
        &mut cursor,
        *remaining_entries,
        remaining_entries,
        |bytes| {
            let mut cursor = Cursor::new(bytes);
            let reference = read_ref(&mut cursor)?;
            let frame = cursor.field()?;
            if frame.len() > format.page_size().bytes() as usize {
                return Err(PhysicalRecoveryProjectionDenial::EntryLimit);
            }
            cursor.end()?;
            Ok(ReleaseCustodyHeadNodeWriteV1::new(
                reference,
                frame.to_vec(),
            ))
        },
    )?;
    cursor.end()?;
    let result_generation = source_root_generation
        .checked_add(1)
        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
    let limits = ReleaseCustodyHeadTransitionLimitsV1::new(
        u16::try_from(path.len().max(1))
            .map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
        u16::try_from(writes.len()).map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
        (path.len() as u64 + writes.len() as u64)
            .checked_mul(u64::from(format.page_size().bytes()))
            .ok_or(PhysicalRecoveryProjectionDenial::EntryLimit)?,
    )
    .ok_or(PhysicalRecoveryProjectionDenial::EntryLimit)?;
    let planned = ReleaseCustodyHeadTransitionV1::verify_exact(
        source_root,
        source_next_block,
        &path,
        mutation,
        result_generation,
        tree_identity,
        format,
        limits,
        Some(result_root),
        result_next_block,
        &writes,
    )
    .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
    PersistedReleaseCustodyHeadEffectV1::new_upsert(
        tree_identity,
        source_root_generation,
        source_basis,
        path,
        planned,
        format,
        limits,
    )
}

fn write_ref(bytes: &mut Vec<u8>, reference: HeadRef) {
    let mut encoded = [0; REF_BYTES];
    reference.encode_into(&mut encoded);
    bytes.extend_from_slice(&encoded);
}

fn read_ref(cursor: &mut Cursor<'_>) -> Result<HeadRef, PhysicalRecoveryProjectionDenial> {
    HeadRef::decode(cursor.take(REF_BYTES)?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)
}

fn write_optional_ref(bytes: &mut Vec<u8>, reference: Option<HeadRef>) {
    bytes.push(u8::from(reference.is_some()));
    if let Some(reference) = reference {
        write_ref(bytes, reference);
    }
}

fn read_optional_ref(
    cursor: &mut Cursor<'_>,
) -> Result<Option<HeadRef>, PhysicalRecoveryProjectionDenial> {
    match cursor.byte()? {
        0 => Ok(None),
        1 => read_ref(cursor).map(Some),
        _ => Err(PhysicalRecoveryProjectionDenial::Malformed),
    }
}

fn write_entry(bytes: &mut Vec<u8>, entry: HeadEntry) {
    let mut encoded = [0; ENTRY_BYTES];
    entry.encode_into(&mut encoded);
    bytes.extend_from_slice(&encoded);
}

fn read_entry(cursor: &mut Cursor<'_>) -> Result<HeadEntry, PhysicalRecoveryProjectionDenial> {
    HeadEntry::decode(cursor.take(ENTRY_BYTES)?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)
}

fn write_optional_entry(bytes: &mut Vec<u8>, entry: Option<HeadEntry>) {
    bytes.push(u8::from(entry.is_some()));
    if let Some(entry) = entry {
        write_entry(bytes, entry);
    }
}

fn read_optional_entry(
    cursor: &mut Cursor<'_>,
) -> Result<Option<HeadEntry>, PhysicalRecoveryProjectionDenial> {
    match cursor.byte()? {
        0 => Ok(None),
        1 => read_entry(cursor).map(Some),
        _ => Err(PhysicalRecoveryProjectionDenial::Malformed),
    }
}
