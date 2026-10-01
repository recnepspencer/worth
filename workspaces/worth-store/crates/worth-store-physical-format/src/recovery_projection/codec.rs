use super::*;

mod cursor;
use cursor::*;
mod domain;
use domain::require_current_domain;
mod frame_coordinate;
use frame_coordinate::{read_subject_coordinate, write_subject_coordinate};
mod head_effect;
use head_effect::{decode_head_effect, encode_head_effect};
#[cfg(test)]
#[path = "codec/head_effect_tests.rs"]
mod head_effect_tests;
mod manifest;
use manifest::{read_manifest, write_manifest};
mod operation;
use operation::{read_operation, write_operation};
mod placement;
use placement::{read_placement, write_placement};
mod segment_update;
use segment_update::{read_segment_update, write_segment_update};
mod sequence;
use sequence::{read_bounded_sequence, read_sequence, write_sequence};
#[cfg(test)]
mod tests;

impl PersistedPhysicalRecoveryProjection {
    pub fn encode(&self) -> Vec<u8> {
        let mut target = Vec::new();
        field(&mut target, CURRENT_RECOVERY_PROJECTION_DOMAIN);
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
        write_operation(&mut target, &self.operation);
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
        require_current_domain(cursor.field()?)?;
        let source_root_generation = cursor.u64()?;
        let root_state =
            PersistedPhysicalRecoveryRootState::decode(cursor.field()?, limits.inline_allocations)
                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        let record_identities = read_sequence(&mut cursor, limits.record_identities, |bytes| {
            let mut item = Cursor::new(bytes);
            let record = read_record(&mut item)?;
            item.end()?;
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
        let operation = read_operation(
            &mut cursor,
            source_root_generation,
            &mut remaining_entries,
            format,
        )?;
        cursor.end()?;
        match payload {
            PersistedPhysicalRecoveryPayload::Frames(frames) => Self::new_with_operation(
                source_root_generation,
                root_state,
                record_identities,
                frames.into_vec(),
                placements,
                segment_updates,
                manifests,
                operation,
            )
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed),
            PersistedPhysicalRecoveryPayload::SourceCopy(recipe) => {
                if operation != PersistedPhysicalRecoveryOperation::None {
                    return Err(PhysicalRecoveryProjectionDenial::Malformed);
                }
                let expected = Self::from_source_copy(source_root_generation, root_state, recipe)
                    .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
                (record_identities.as_slice() == expected.record_identities()
                    && placements.as_slice() == expected.placements()
                    && segment_updates.is_empty()
                    && manifests.is_empty())
                .then_some(expected)
                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)
            }
        }
    }
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
