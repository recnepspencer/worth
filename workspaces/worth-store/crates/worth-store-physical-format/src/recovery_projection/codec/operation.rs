use super::*;

const MAXIMUM_OPERATION_BINDING_BYTES: usize = 1024;

pub(super) fn write_operation(
    target: &mut Vec<u8>,
    operation: &PersistedPhysicalRecoveryOperation,
) {
    let mut binding_bytes = Vec::with_capacity(160);
    let binding = match operation {
        PersistedPhysicalRecoveryOperation::None => {
            binding_bytes.push(0);
            field(target, &binding_bytes);
            return;
        }
        PersistedPhysicalRecoveryOperation::SessionDeclared(binding) => {
            binding_bytes.push(1);
            *binding
        }
        PersistedPhysicalRecoveryOperation::GenerationPublished(binding) => {
            binding_bytes.push(2);
            *binding
        }
        PersistedPhysicalRecoveryOperation::SessionFrontier(binding) => {
            binding_bytes.push(3);
            *binding
        }
        PersistedPhysicalRecoveryOperation::SessionAbandoned(binding) => {
            binding_bytes.push(4);
            *binding
        }
        PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } => {
            binding_bytes.push(5);
            *binding
        }
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding,
            retirement,
        } => {
            binding_bytes.push(6);
            write_binding(&mut binding_bytes, binding.record());
            write_publication(&mut binding_bytes, binding.indexed_through());
            match binding.indexed_through_quarantine() {
                None => binding_bytes.push(0),
                Some(None) => binding_bytes.push(1),
                Some(Some(record)) => {
                    binding_bytes.push(2);
                    write_record(&mut binding_bytes, record);
                }
            }
            field(target, &binding_bytes);
            if let Some(retirement) = retirement {
                target.push(1);
                write_previous(target, retirement.expected_previous());
                write_sequence(target, retirement.dropped_records(), |out, record| {
                    write_record(out, *record)
                });
            } else {
                target.push(0);
            }
            return;
        }
        PersistedPhysicalRecoveryOperation::ChunkReused(binding) => {
            binding_bytes.push(7);
            *binding
        }
        PersistedPhysicalRecoveryOperation::DedupeQuarantined(binding) => {
            binding_bytes.push(8);
            *binding
        }
        PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(retirement) => {
            binding_bytes.push(9);
            write_record(&mut binding_bytes, retirement.declaration_record());
            binding_bytes.extend_from_slice(&retirement.declaration_frame_sha256());
            field(target, &binding_bytes);
            field(target, &encode_terminal_head_retirement(retirement));
            return;
        }
    };
    write_binding(&mut binding_bytes, binding);
    field(target, &binding_bytes);
    if let PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect,
        directory_replacement,
        ..
    } = operation
    {
        if let Some(effect) = head_effect {
            target.push(1);
            field(target, &encode_head_effect(effect));
        } else {
            target.push(0);
        }
        if let Some(replacement) = directory_replacement {
            target.push(1);
            write_previous(target, Some(replacement.expected_previous()));
            target.extend_from_slice(&replacement.expected_previous_payload_sha256());
            let next = replacement.next();
            write_binding(target, next.record());
            write_publication(target, next.indexed_through());
            match next.indexed_through_quarantine() {
                None => target.push(0),
                Some(None) => target.push(1),
                Some(Some(record)) => {
                    target.push(2);
                    write_record(target, record);
                }
            }
        } else {
            target.push(0);
        }
    }
}

pub(super) fn read_operation<S: PhysicalRecoveryDecodeStorage>(
    cursor: &mut Cursor<'_>,
    source_root_generation: u64,
    remaining_entries: &mut u64,
    format: Option<crate::PhysicalRecordFormatDeclaration>,
    storage: &mut S,
) -> Result<PersistedPhysicalRecoveryOperation, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let bytes = cursor.field()?;
    if bytes.len() > MAXIMUM_OPERATION_BINDING_BYTES {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit.into());
    }
    let mut binding = Cursor::new(bytes);
    let operation = match binding.byte()? {
        0 => PersistedPhysicalRecoveryOperation::None,
        tag @ (1 | 2 | 3 | 4 | 5 | 7 | 8) => {
            let value = read_binding(&mut binding)?;
            match tag {
                1 => PersistedPhysicalRecoveryOperation::SessionDeclared(value),
                2 => PersistedPhysicalRecoveryOperation::GenerationPublished(value),
                3 => PersistedPhysicalRecoveryOperation::SessionFrontier(value),
                4 => PersistedPhysicalRecoveryOperation::SessionAbandoned(value),
                5 => {
                    let head_effect = match cursor.byte()? {
                        0 => None,
                        1 => Some(decode_head_effect(
                            cursor.field()?,
                            source_root_generation,
                            remaining_entries,
                            format.ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
                            storage,
                        )?),
                        _ => return Err(PhysicalRecoveryProjectionDenial::Malformed.into()),
                    };
                    let directory_replacement = match cursor.byte()? {
                        0 => None,
                        1 => {
                            let previous = read_previous(cursor)?
                                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
                            let previous_digest = cursor
                                .take(32)?
                                .try_into()
                                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
                            let next_record = read_binding(cursor)?;
                            let publication = read_publication(cursor)?;
                            let next = match cursor.byte()? {
                                0 => PersistedDerivedDirectoryRecordBinding::new(
                                    next_record,
                                    publication,
                                ),
                                1 => PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                                    next_record,
                                    publication,
                                    None,
                                ),
                                2 => PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                                    next_record,
                                    publication,
                                    Some(read_record(cursor)?),
                                ),
                                _ => return Err(PhysicalRecoveryProjectionDenial::Malformed.into()),
                            };
                            Some(
                                PersistedReleasedDirectoryReplacementV1::new(
                                    previous,
                                    previous_digest,
                                    next,
                                )
                                .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
                            )
                        }
                        _ => return Err(PhysicalRecoveryProjectionDenial::Malformed.into()),
                    };
                    PersistedPhysicalRecoveryOperation::RecordsDropped {
                        binding: value,
                        head_effect,
                        directory_replacement,
                    }
                }
                7 => PersistedPhysicalRecoveryOperation::ChunkReused(value),
                8 => PersistedPhysicalRecoveryOperation::DedupeQuarantined(value),
                _ => return Err(PhysicalRecoveryProjectionDenial::Malformed.into()),
            }
        }
        6 => {
            let value = read_binding(&mut binding)?;
            let publication = read_publication(&mut binding)?;
            let directory = match binding.byte()? {
                0 => PersistedDerivedDirectoryRecordBinding::new(value, publication),
                1 => PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                    value,
                    publication,
                    None,
                ),
                2 => PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                    value,
                    publication,
                    Some(read_record(&mut binding)?),
                ),
                _ => return Err(PhysicalRecoveryProjectionDenial::Malformed.into()),
            };
            let retirement = match cursor.byte()? {
                0 => None,
                1 => {
                    let previous = read_previous(cursor)?;
                    let dropped = read_bounded_sequence(
                        cursor,
                        *remaining_entries,
                        remaining_entries,
                        storage,
                        |bytes, _| {
                            let mut item = Cursor::new(bytes);
                            let record = read_record(&mut item)?;
                            item.end()?;
                            Ok(record)
                        },
                    )?;
                    Some(
                        PersistedDerivedDirectoryRetirement::new(previous, dropped)
                            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
                    )
                }
                _ => return Err(PhysicalRecoveryProjectionDenial::Malformed.into()),
            };
            PersistedPhysicalRecoveryOperation::DerivedDirectory {
                binding: directory,
                retirement,
            }
        }
        9 => {
            let declaration_record = read_record(&mut binding)?;
            let declaration_frame_sha256 = binding
                .take(32)?
                .try_into()
                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
            PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(
                decode_terminal_head_retirement(
                    cursor.field()?,
                    declaration_record,
                    declaration_frame_sha256,
                    source_root_generation,
                    remaining_entries,
                    format.ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
                    storage,
                )?,
            )
        }
        _ => return Err(PhysicalRecoveryProjectionDenial::Malformed.into()),
    };
    binding.end()?;
    Ok(operation)
}

fn write_binding(target: &mut Vec<u8>, binding: PersistedBlobSemanticRecordBinding) {
    write_record(target, binding.record());
    target.extend_from_slice(&binding.record_payload_sha256());
    target.extend_from_slice(&binding.candidate_root_generation().to_le_bytes());
}

fn read_binding(
    cursor: &mut Cursor<'_>,
) -> Result<PersistedBlobSemanticRecordBinding, PhysicalRecoveryProjectionDenial> {
    let record = read_record(cursor)?;
    let digest = cursor
        .take(32)?
        .try_into()
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
    PersistedBlobSemanticRecordBinding::new(record, digest, cursor.u64()?)
        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)
}

fn write_publication(target: &mut Vec<u8>, source: Option<crate::IndexedThroughBlobPublication>) {
    if let Some(source) = source {
        target.push(1);
        write_record(target, source.record());
        target.extend_from_slice(&source.root_generation().to_le_bytes());
        target.extend_from_slice(&source.encoded_digest());
    } else {
        target.push(0);
    }
}

fn read_publication(
    cursor: &mut Cursor<'_>,
) -> Result<Option<crate::IndexedThroughBlobPublication>, PhysicalRecoveryProjectionDenial> {
    match cursor.byte()? {
        0 => Ok(None),
        1 => {
            let record = read_record(cursor)?;
            let generation = cursor.u64()?;
            let digest = cursor
                .take(32)?
                .try_into()
                .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
            Ok(Some(
                crate::IndexedThroughBlobPublication::new(generation, record, digest)
                    .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?,
            ))
        }
        _ => Err(PhysicalRecoveryProjectionDenial::Malformed),
    }
}

fn write_previous(
    target: &mut Vec<u8>,
    previous: Option<crate::DerivedFamilyRootDirectoryBinding>,
) {
    if let Some(previous) = previous {
        target.push(1);
        write_record(target, previous.directory_record());
        write_publication(target, previous.indexed_through_blob_publication());
    } else {
        target.push(0);
    }
}

fn read_previous(
    cursor: &mut Cursor<'_>,
) -> Result<Option<crate::DerivedFamilyRootDirectoryBinding>, PhysicalRecoveryProjectionDenial> {
    match cursor.byte()? {
        0 => Ok(None),
        1 => {
            let record = read_record(cursor)?;
            Ok(Some(crate::DerivedFamilyRootDirectoryBinding::new(
                record,
                read_publication(cursor)?,
            )))
        }
        _ => Err(PhysicalRecoveryProjectionDenial::Malformed),
    }
}
