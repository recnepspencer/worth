use super::*;

mod cursor;
use cursor::*;
mod blob_semantic;
use blob_semantic::{decode_blob_semantic, encode_blob_semantic};
#[cfg(test)]
#[path = "codec/head_effect_tests.rs"]
mod head_effect_tests;
mod manifest;
#[cfg(test)]
mod tests;
use manifest::{read_manifest, write_manifest};
mod sequence;
use sequence::{read_bounded_sequence, read_sequence, write_sequence};
mod frame_coordinate;
use frame_coordinate::{read_subject_coordinate, write_subject_coordinate};
mod head_effect;
use head_effect::{decode_head_effect, encode_head_effect};
mod placement;
use placement::{read_placement, write_placement};
mod segment_update;
use segment_update::{read_segment_update, write_segment_update};
mod version_semantics;

impl PersistedPhysicalRecoveryProjection {
    pub fn encode(&self) -> Vec<u8> {
        let mut target = Vec::new();
        field(
            &mut target,
            match self.version {
                RecoveryProjectionVersion::V5 => V5_DOMAIN,
                RecoveryProjectionVersion::V6 => V6_DOMAIN,
                RecoveryProjectionVersion::V7 => V7_DOMAIN,
                RecoveryProjectionVersion::V8 => V8_DOMAIN,
                RecoveryProjectionVersion::V9 => V9_DOMAIN,
                RecoveryProjectionVersion::V10 => V10_DOMAIN,
                RecoveryProjectionVersion::V11 => V11_DOMAIN,
                RecoveryProjectionVersion::V12 => V12_DOMAIN,
                RecoveryProjectionVersion::V13 => V13_DOMAIN,
                RecoveryProjectionVersion::V14 => V14_DOMAIN,
            },
        );
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
        if self.version != RecoveryProjectionVersion::V5 {
            field(
                &mut target,
                &encode_blob_semantic(self.blob_semantic, self.version),
            );
        }
        if matches!(
            self.version,
            RecoveryProjectionVersion::V13 | RecoveryProjectionVersion::V14
        ) {
            target.push(u8::from(self.derived_retirement.is_some()));
        }
        if let Some(retirement) = &self.derived_retirement {
            match retirement.expected_previous() {
                Some(binding) => {
                    target.push(1);
                    write_record(&mut target, binding.directory_record());
                    match binding.indexed_through_blob_publication() {
                        Some(source) => {
                            target.push(1);
                            write_record(&mut target, source.record());
                            target.extend_from_slice(&source.root_generation().to_le_bytes());
                            target.extend_from_slice(&source.encoded_digest());
                        }
                        None => target.push(0),
                    }
                }
                None => target.push(0),
            }
            write_sequence(
                &mut target,
                retirement.dropped_records(),
                |target, record| write_record(target, *record),
            );
        }
        write_sequence(&mut target, &self.placements, |target, placement| {
            write_placement(
                target,
                placement,
                matches!(
                    self.version,
                    RecoveryProjectionVersion::V13 | RecoveryProjectionVersion::V14
                ),
            )
        });
        write_sequence(&mut target, &self.segment_updates, write_segment_update);
        write_sequence(&mut target, &self.manifests, write_manifest);
        if self.version == RecoveryProjectionVersion::V14 {
            field(
                &mut target,
                &encode_head_effect(
                    self.release_head_effect
                        .as_ref()
                        .expect("V14 has one release-head effect"),
                ),
            );
        }
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
        let domain = cursor.field()?;
        let version = if domain == V5_DOMAIN {
            RecoveryProjectionVersion::V5
        } else if domain == V6_DOMAIN {
            RecoveryProjectionVersion::V6
        } else if domain == V7_DOMAIN {
            RecoveryProjectionVersion::V7
        } else if domain == V8_DOMAIN {
            RecoveryProjectionVersion::V8
        } else if domain == V9_DOMAIN {
            RecoveryProjectionVersion::V9
        } else if domain == V10_DOMAIN {
            RecoveryProjectionVersion::V10
        } else if domain == V11_DOMAIN {
            RecoveryProjectionVersion::V11
        } else if domain == V12_DOMAIN {
            RecoveryProjectionVersion::V12
        } else if domain == V13_DOMAIN {
            RecoveryProjectionVersion::V13
        } else if domain == V14_DOMAIN {
            RecoveryProjectionVersion::V14
        } else {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        };
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
        let blob_semantic = match version {
            RecoveryProjectionVersion::V5 => PersistedPhysicalRecoveryBlobSemantic::None,
            RecoveryProjectionVersion::V6
            | RecoveryProjectionVersion::V7
            | RecoveryProjectionVersion::V8
            | RecoveryProjectionVersion::V9
            | RecoveryProjectionVersion::V10
            | RecoveryProjectionVersion::V11
            | RecoveryProjectionVersion::V12 => decode_blob_semantic(cursor.field()?, version)?,
            RecoveryProjectionVersion::V13 | RecoveryProjectionVersion::V14 => {
                decode_blob_semantic(cursor.field()?, version)?
            }
        };
        let mut remaining_entries = limits.total_entries;
        let has_retirement = if matches!(
            version,
            RecoveryProjectionVersion::V13 | RecoveryProjectionVersion::V14
        ) {
            match cursor.byte()? {
                0 => false,
                1 => true,
                _ => return Err(PhysicalRecoveryProjectionDenial::Malformed),
            }
        } else {
            matches!(
                version,
                RecoveryProjectionVersion::V10 | RecoveryProjectionVersion::V12
            )
        };
        let retirement = if has_retirement {
            let expected_previous = match cursor.byte()? {
                0 => None,
                1 => {
                    let directory_record = read_record(&mut cursor)?;
                    let source = match cursor.byte()? {
                        0 => None,
                        1 => {
                            let record = read_record(&mut cursor)?;
                            let generation = cursor.u64()?;
                            let digest = cursor
                                .take(32)?
                                .try_into()
                                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
                            Some(
                                crate::IndexedThroughBlobPublication::new(
                                    generation, record, digest,
                                )
                                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
                            )
                        }
                        _ => return Err(PhysicalRecoveryProjectionDenial::Malformed),
                    };
                    Some(crate::DerivedFamilyRootDirectoryBinding::new(
                        directory_record,
                        source,
                    ))
                }
                _ => return Err(PhysicalRecoveryProjectionDenial::Malformed),
            };
            let dropped = read_bounded_sequence(
                &mut cursor,
                limits.total_entries,
                &mut remaining_entries,
                |bytes| {
                    let mut cursor = Cursor::new(bytes);
                    let record = read_record(&mut cursor)?;
                    cursor.end()?;
                    Ok(record)
                },
            )?;
            Some((expected_previous, dropped))
        } else {
            None
        };
        let placements = read_bounded_sequence(
            &mut cursor,
            limits.placements,
            &mut remaining_entries,
            |bytes| {
                read_placement(
                    bytes,
                    matches!(
                        version,
                        RecoveryProjectionVersion::V13 | RecoveryProjectionVersion::V14
                    ),
                )
            },
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
        let release_head_effect = if version == RecoveryProjectionVersion::V14 {
            if has_retirement {
                return Err(PhysicalRecoveryProjectionDenial::Malformed);
            }
            Some(decode_head_effect(
                cursor.field()?,
                source_root_generation,
                &mut remaining_entries,
                format.ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
            )?)
        } else {
            None
        };
        cursor.end()?;
        let mut projection = match payload {
            PersistedPhysicalRecoveryPayload::Frames(frames) => Self::new_with_blob_semantic(
                source_root_generation,
                root_state,
                record_identities,
                frames.into_vec(),
                placements,
                segment_updates,
                manifests,
                blob_semantic,
            ),
            PersistedPhysicalRecoveryPayload::SourceCopy(recipe) => {
                if blob_semantic != PersistedPhysicalRecoveryBlobSemantic::None {
                    return Err(PhysicalRecoveryProjectionDenial::Malformed);
                }
                let expected = Self::from_source_copy(source_root_generation, root_state, recipe)
                    .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
                (record_identities.as_slice() == expected.record_identities()
                    && placements.as_slice() == expected.placements()
                    && segment_updates.is_empty()
                    && manifests.is_empty())
                .then_some(expected)
            }
        }
        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        version_semantics::admit(version, blob_semantic)?;
        if let Some((expected_previous, dropped)) = retirement {
            projection = projection
                .with_derived_retirement(expected_previous, dropped)
                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        }
        if let Some(effect) = release_head_effect {
            projection = projection
                .with_release_head_upsert(effect)
                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        }
        projection.version = version;
        Ok(projection)
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
