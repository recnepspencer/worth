use super::*;

const MAXIMUM_BLOB_SEMANTIC_BYTES: usize = 1024;

pub(super) fn encode_blob_semantic(
    semantic: PersistedPhysicalRecoveryBlobSemantic,
    version: RecoveryProjectionVersion,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(65);
    let binding = match semantic {
        PersistedPhysicalRecoveryBlobSemantic::None => {
            bytes.push(0);
            return bytes;
        }
        PersistedPhysicalRecoveryBlobSemantic::SessionDeclared(binding) => {
            bytes.push(1);
            binding
        }
        PersistedPhysicalRecoveryBlobSemantic::GenerationPublished(binding) => {
            bytes.push(2);
            binding
        }
        PersistedPhysicalRecoveryBlobSemantic::SessionFrontier(binding) => {
            bytes.push(3);
            binding
        }
        PersistedPhysicalRecoveryBlobSemantic::SessionAbandoned(binding) => {
            bytes.push(4);
            binding
        }
        PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding) => {
            bytes.push(5);
            binding
        }
        PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(directory) => {
            bytes.push(6);
            let binding = directory.record();
            write_record(&mut bytes, binding.record());
            bytes.extend_from_slice(&binding.record_payload_sha256());
            bytes.extend_from_slice(&binding.candidate_root_generation().to_le_bytes());
            if let Some(publication) = directory.indexed_through() {
                bytes.push(1);
                write_record(&mut bytes, publication.record());
                bytes.extend_from_slice(&publication.root_generation().to_le_bytes());
                bytes.extend_from_slice(&publication.encoded_digest());
            } else {
                bytes.extend_from_slice(&[0; 65]);
            }
            if let Some(quarantine) = directory.indexed_through_quarantine() {
                match quarantine {
                    Some(record) => {
                        bytes.push(1);
                        write_record(&mut bytes, record);
                    }
                    None => bytes.extend_from_slice(&[0; 25]),
                }
            } else if matches!(
                version,
                RecoveryProjectionVersion::V13 | RecoveryProjectionVersion::V14
            ) {
                bytes.extend_from_slice(&[0; 25]);
            }
            return bytes;
        }
        PersistedPhysicalRecoveryBlobSemantic::ChunkReused(binding) => {
            bytes.push(7);
            binding
        }
        PersistedPhysicalRecoveryBlobSemantic::DedupeQuarantined(binding) => {
            bytes.push(8);
            binding
        }
    };
    write_record(&mut bytes, binding.record());
    bytes.extend_from_slice(&binding.record_payload_sha256());
    bytes.extend_from_slice(&binding.candidate_root_generation().to_le_bytes());
    bytes
}

pub(super) fn decode_blob_semantic(
    bytes: &[u8],
    version: RecoveryProjectionVersion,
) -> Result<PersistedPhysicalRecoveryBlobSemantic, PhysicalRecoveryProjectionDenial> {
    if bytes.len() > MAXIMUM_BLOB_SEMANTIC_BYTES {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit);
    }
    let mut cursor = Cursor::new(bytes);
    let semantic = match cursor.byte()? {
        0 => PersistedPhysicalRecoveryBlobSemantic::None,
        kind @ (1 | 2 | 3 | 4 | 5) => {
            if kind == 5
                && !matches!(
                    version,
                    RecoveryProjectionVersion::V7
                        | RecoveryProjectionVersion::V13
                        | RecoveryProjectionVersion::V14
                )
            {
                return Err(PhysicalRecoveryProjectionDenial::Malformed);
            }
            let record = read_record(&mut cursor)?;
            let digest = cursor
                .take(32)?
                .try_into()
                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
            let root = cursor.u64()?;
            let binding = PersistedBlobSemanticRecordBinding::new(record, digest, root)
                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
            match kind {
                1 => PersistedPhysicalRecoveryBlobSemantic::SessionDeclared(binding),
                2 => PersistedPhysicalRecoveryBlobSemantic::GenerationPublished(binding),
                3 => PersistedPhysicalRecoveryBlobSemantic::SessionFrontier(binding),
                4 => PersistedPhysicalRecoveryBlobSemantic::SessionAbandoned(binding),
                5 => PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding),
                _ => unreachable!("known blob semantic tag"),
            }
        }
        6 if matches!(
            version,
            RecoveryProjectionVersion::V8
                | RecoveryProjectionVersion::V10
                | RecoveryProjectionVersion::V12
                | RecoveryProjectionVersion::V13
                | RecoveryProjectionVersion::V14
        ) =>
        {
            let record = read_record(&mut cursor)?;
            let digest = cursor
                .take(32)?
                .try_into()
                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
            let root = cursor.u64()?;
            let binding = PersistedBlobSemanticRecordBinding::new(record, digest, root)
                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
            let publication = match cursor.byte()? {
                0 => {
                    if cursor.take(64)? != [0; 64] {
                        return Err(PhysicalRecoveryProjectionDenial::Malformed);
                    }
                    None
                }
                1 => {
                    let record = read_record(&mut cursor)?;
                    let generation = cursor.u64()?;
                    let digest = cursor
                        .take(32)?
                        .try_into()
                        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
                    Some(
                        crate::IndexedThroughBlobPublication::new(generation, record, digest)
                            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
                    )
                }
                _ => return Err(PhysicalRecoveryProjectionDenial::Malformed),
            };
            let directory = if matches!(
                version,
                RecoveryProjectionVersion::V12
                    | RecoveryProjectionVersion::V13
                    | RecoveryProjectionVersion::V14
            ) {
                let quarantine = match cursor.byte()? {
                    0 => {
                        if cursor.take(24)? != [0; 24] {
                            return Err(PhysicalRecoveryProjectionDenial::Malformed);
                        }
                        None
                    }
                    1 => Some(read_record(&mut cursor)?),
                    _ => return Err(PhysicalRecoveryProjectionDenial::Malformed),
                };
                PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                    binding,
                    publication,
                    quarantine,
                )
            } else {
                PersistedDerivedDirectoryRecordBinding::new(binding, publication)
            };
            PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(directory)
        }
        7 if matches!(
            version,
            RecoveryProjectionVersion::V9
                | RecoveryProjectionVersion::V13
                | RecoveryProjectionVersion::V14
        ) =>
        {
            let record = read_record(&mut cursor)?;
            let digest = cursor
                .take(32)?
                .try_into()
                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
            let root = cursor.u64()?;
            PersistedPhysicalRecoveryBlobSemantic::ChunkReused(
                PersistedBlobSemanticRecordBinding::new(record, digest, root)
                    .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
            )
        }
        8 if matches!(
            version,
            RecoveryProjectionVersion::V11
                | RecoveryProjectionVersion::V13
                | RecoveryProjectionVersion::V14
        ) =>
        {
            let record = read_record(&mut cursor)?;
            let digest = cursor
                .take(32)?
                .try_into()
                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
            let root = cursor.u64()?;
            PersistedPhysicalRecoveryBlobSemantic::DedupeQuarantined(
                PersistedBlobSemanticRecordBinding::new(record, digest, root)
                    .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
            )
        }
        _ => return Err(PhysicalRecoveryProjectionDenial::Malformed),
    };
    cursor.end()?;
    Ok(semantic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_semantic_is_canonical_and_bounded_independently_of_projection_limits() {
        assert_eq!(
            decode_blob_semantic(&[0], RecoveryProjectionVersion::V6),
            Ok(PersistedPhysicalRecoveryBlobSemantic::None),
        );
        assert_eq!(
            decode_blob_semantic(&[0, 0], RecoveryProjectionVersion::V6),
            Err(PhysicalRecoveryProjectionDenial::Malformed),
        );
        assert_eq!(
            decode_blob_semantic(&[3], RecoveryProjectionVersion::V6),
            Err(PhysicalRecoveryProjectionDenial::Malformed),
        );
        assert_eq!(
            decode_blob_semantic(
                &vec![0; MAXIMUM_BLOB_SEMANTIC_BYTES + 1],
                RecoveryProjectionVersion::V6
            ),
            Err(PhysicalRecoveryProjectionDenial::EntryLimit),
        );
    }
}
