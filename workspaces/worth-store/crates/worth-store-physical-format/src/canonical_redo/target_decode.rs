use std::collections::BTreeSet;

use super::{
    CanonicalRedoExtentCoordinate, CanonicalRedoTarget, CanonicalRedoTargetIdentity,
    CanonicalRedoWireDenial, Cursor,
};
use crate::recovery_projection::decode_storage::reserve_vec;
use crate::RecordArtifactFile;
use crate::{PhysicalRecoveryDecodeFailure, PhysicalRecoveryDecodeStorage};

pub(super) fn decode_targets<S: PhysicalRecoveryDecodeStorage>(
    cursor: &mut Cursor<'_>,
    total: &mut u64,
    maximum: u64,
    distinct: &mut Option<(&mut BTreeSet<CanonicalRedoTargetIdentity>, u64)>,
    storage: &mut S,
) -> Result<Box<[CanonicalRedoTarget]>, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let count = cursor.u64()?;
    if count == 0 {
        return Err(CanonicalRedoWireDenial::InvalidTarget.into());
    }
    *total = total
        .checked_add(count)
        .ok_or(CanonicalRedoWireDenial::CounterOverflow)?;
    if *total > maximum {
        return Err(CanonicalRedoWireDenial::TargetLimit.into());
    }
    cursor.require_count_backing(count, 8 + 32 + 1)?;
    let mut targets = reserve_vec(
        usize::try_from(count).map_err(|_| CanonicalRedoWireDenial::TargetLimit)?,
        storage,
    )?;
    let mut prior = None;
    for _ in 0..count {
        let encoded = cursor.field()?;
        let resulting_digest = cursor.array()?;
        let target = decode_target(encoded, resulting_digest)?;
        if let Some((identities, maximum_distinct)) = distinct.as_mut() {
            let identity = target.identity();
            if !identities.contains(&identity) && identities.len() as u64 == *maximum_distinct {
                return Err(CanonicalRedoWireDenial::DistinctTargetLimit.into());
            }
            identities.insert(identity);
        }
        let order = target.canonical_order();
        if prior.as_ref().is_some_and(|prior| prior > &order) {
            return Err(CanonicalRedoWireDenial::NonCanonicalTargetOrder.into());
        }
        prior = Some(order);
        targets.push(target);
    }
    Ok(targets.into_boxed_slice())
}

fn decode_target(
    encoded: &[u8],
    resulting_digest: [u8; 32],
) -> Result<CanonicalRedoTarget, CanonicalRedoWireDenial> {
    let mut cursor = Cursor::new(encoded);
    let kind = cursor.byte()?;
    let (identity, extent_coordinate) = match kind {
        1 => (decode_inline_identity(&mut cursor)?, None),
        2 => {
            let (identity, coordinate) = decode_extent_identity(&mut cursor)?;
            (identity, Some(coordinate))
        }
        _ => return Err(CanonicalRedoWireDenial::InvalidTarget),
    };
    let artifact = cursor.byte()?;
    let artifact_identity = cursor.u64()?;
    let artifact_generation = if artifact == 16 { 0 } else { cursor.u64()? };
    let artifact_offset = cursor.u64()?;
    let artifact_length = cursor.u32()?;
    cursor.require_end()?;
    let artifact = match identity {
        CanonicalRedoTargetIdentity::InlinePage { segment, .. }
            if artifact == 5 && artifact_identity == segment =>
        {
            RecordArtifactFile::Segment {
                segment,
                generation: artifact_generation,
            }
        }
        CanonicalRedoTargetIdentity::ExtentChunk { .. }
            if matches!(artifact, 8 | 16) && artifact_identity != 0 && artifact_generation == 0 =>
        {
            RecordArtifactFile::ExtentArena {
                arena: artifact_identity,
            }
        }
        _ => return Err(CanonicalRedoWireDenial::InvalidTarget),
    };
    if artifact_length == 0 {
        return Err(CanonicalRedoWireDenial::InvalidTarget);
    }
    Ok(CanonicalRedoTarget {
        identity,
        extent_coordinate,
        artifact,
        artifact_offset,
        artifact_length,
        resulting_digest,
    })
}

fn decode_inline_identity(
    cursor: &mut Cursor<'_>,
) -> Result<CanonicalRedoTargetIdentity, CanonicalRedoWireDenial> {
    let segment = cursor.u64()?;
    let page = cursor.u64()?;
    let generation = cursor.u64()?;
    if segment == 0 || page == 0 || generation == 0 {
        return Err(CanonicalRedoWireDenial::InvalidTarget);
    }
    Ok(CanonicalRedoTargetIdentity::InlinePage {
        segment,
        page,
        generation,
    })
}

fn decode_extent_identity(
    cursor: &mut Cursor<'_>,
) -> Result<(CanonicalRedoTargetIdentity, CanonicalRedoExtentCoordinate), CanonicalRedoWireDenial> {
    let allocation_epoch = cursor.array()?;
    let record_ordinal = cursor.u64()?;
    let extent = cursor.u64()?;
    let generation = cursor.u64()?;
    let logical_bytes = cursor.u64()?;
    let logical_offset = cursor.u64()?;
    let chunk = cursor.u32()?;
    if allocation_epoch == [0; 16]
        || record_ordinal == 0
        || extent == 0
        || generation == 0
        || logical_bytes == 0
        || logical_offset >= logical_bytes
        || chunk == 0
    {
        return Err(CanonicalRedoWireDenial::InvalidTarget);
    }
    Ok((
        CanonicalRedoTargetIdentity::ExtentChunk {
            extent,
            generation,
            chunk,
        },
        CanonicalRedoExtentCoordinate {
            allocation_epoch,
            record_ordinal,
            logical_bytes,
            logical_offset,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal_extent_target(tag: u8) -> Vec<u8> {
        let mut bytes = vec![2];
        bytes.extend_from_slice(&[7; 16]);
        for number in [1_u64, 2, 3, 40_000, 0] {
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.push(tag);
        bytes.extend_from_slice(&9_u64.to_le_bytes());
        if tag == 8 {
            bytes.extend_from_slice(&0_u64.to_le_bytes());
        }
        bytes.extend_from_slice(&4096_u64.to_le_bytes());
        bytes.extend_from_slice(&16384_u32.to_le_bytes());
        bytes
    }

    #[test]
    fn current_arena_tag_consumes_one_identity_word_and_legacy_tag_two() {
        let current = literal_extent_target(16);
        let legacy = literal_extent_target(8);
        assert_eq!(current.len(), 82);
        assert_eq!(legacy.len(), 90);
        for encoded in [&current, &legacy] {
            let target = decode_target(encoded, [5; 32]).unwrap();
            assert_eq!(
                target.artifact(),
                RecordArtifactFile::ExtentArena { arena: 9 }
            );
            assert_eq!(target.artifact_offset(), 4096);
            assert_eq!(target.artifact_length(), 16384);
            let mut framed = Vec::new();
            framed.extend_from_slice(&1_u64.to_le_bytes());
            framed.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
            framed.extend_from_slice(encoded);
            framed.extend_from_slice(&[5; 32]);
            let mut cursor = Cursor::new(&framed);
            let mut total = 0;
            let decoded = decode_targets(
                &mut cursor,
                &mut total,
                1,
                &mut None,
                &mut crate::recovery_projection::decode_storage::UnrestrictedDecodeStorage,
            )
            .unwrap();
            assert_eq!(decoded.len(), 1);
            assert_eq!(total, 1);
            cursor.require_end().unwrap();
        }
        let mut trailing = current;
        trailing.push(0);
        assert_eq!(
            decode_target(&trailing, [5; 32]),
            Err(CanonicalRedoWireDenial::MalformedMember)
        );
    }
}
