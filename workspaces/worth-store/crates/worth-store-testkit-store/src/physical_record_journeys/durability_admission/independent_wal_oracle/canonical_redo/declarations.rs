pub use {
    crate::durability_admission::independent_wal_oracle::target_claim::IndependentRedoTargetClaim,
    crate::durability_admission::independent_wal_oracle::BindingField,
    crate::durability_admission::independent_wal_oracle::BindingInspectionDenial,
    crate::durability_admission::independent_wal_oracle::ByteCursor,
};

pub const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
pub const CURRENT_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v16";

pub fn independent_canonical_redo(
    records: &[&[u8]],
    lsn_start: u64,
    targets: &[Vec<IndependentRedoTargetClaim>],
    projection: &[u8],
) -> Vec<u8> {
    assert!(
        !records.is_empty(),
        "the independent redo oracle requires a nonempty fixture"
    );
    assert_eq!(
        records.len(),
        targets.len(),
        "every redo record requires its exact target claims"
    );
    let mut encoded = Vec::new();
    write_field(&mut encoded, REDO_DOMAIN);
    encoded.extend_from_slice(&(records.len() as u64).to_le_bytes());
    for (ordinal, record) in records.iter().enumerate() {
        encoded.extend_from_slice(&(ordinal as u32).to_le_bytes());
        encoded.extend_from_slice(&(lsn_start + ordinal as u64).to_le_bytes());
        encoded.extend_from_slice(&(targets[ordinal].len() as u64).to_le_bytes());
        for claim in &targets[ordinal] {
            write_field(&mut encoded, &claim.target);
            encoded.extend_from_slice(&claim.digest);
        }
        write_field(&mut encoded, record);
    }
    write_field(&mut encoded, projection);
    encoded
}

pub fn independent_recovery_projection(
    canonical_redo: &[u8],
) -> Result<&[u8], BindingInspectionDenial> {
    let mut cursor =
        crate::durability_admission::independent_wal_oracle::ByteCursor::new(canonical_redo);
    if cursor
        .field(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload)?
        != REDO_DOMAIN
    {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch);
    }
    let record_count = cursor.read_u64()?;
    if record_count == 0 {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    for _ in 0..record_count {
        cursor.take(
            crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
            4,
        )?;
        cursor.take(
            crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
            8,
        )?;
        let target_count = cursor.read_u64()?;
        if target_count == 0 {
            return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
        }
        for _ in 0..target_count {
            cursor.field(
                crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
            )?;
            cursor.take(
                crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
                32,
            )?;
        }
        cursor.field(
            crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
        )?;
    }
    let projection = cursor
        .field(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload)?;
    cursor.finish()?;

    inspect_recovery_projection(projection)?;
    Ok(projection)
}

pub fn inspect_recovery_projection(projection: &[u8]) -> Result<(), BindingInspectionDenial> {
    let mut cursor =
        crate::durability_admission::independent_wal_oracle::ByteCursor::new(projection);
    let domain = cursor
        .field(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload)?;
    if domain != CURRENT_PROJECTION_DOMAIN {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch);
    }
    if cursor.read_u64()? == 0
        || cursor
            .field(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload)?
            .is_empty()
    {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    read_sequence(&mut cursor)?; // selected record identities
    let source_copy = match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
        [0] => {
            read_sequence(&mut cursor)?; // physical frames
            false
        }
        [1] => {
            cursor.field(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload)?; // embedded copy intent
            cursor.read_u64()?; // intent LSN
            cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 32)?; // intent digest
            true
        }
        _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
    };
    read_sequence(&mut cursor)?; // placements
    read_sequence(&mut cursor)?; // segment updates
    read_sequence(&mut cursor)?; // manifests
    inspect_operation(&mut cursor, source_copy)?;
    cursor.finish()
}

pub fn read_sequence(cursor: &mut ByteCursor<'_>) -> Result<(), BindingInspectionDenial> {
    let count = cursor.read_u64()?;
    for _ in 0..count {
        cursor.field(
            crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
        )?;
    }
    Ok(())
}

pub fn inspect_operation(
    cursor: &mut ByteCursor<'_>,
    source_copy: bool,
) -> Result<(), BindingInspectionDenial> {
    let bytes = cursor
        .field(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload)?;
    let Some((&tag, binding)) = bytes.split_first() else {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    };
    if tag == 0 {
        return (binding.is_empty())
            .then_some(())
            .ok_or(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    if source_copy || !matches!(tag, 1..=8) || binding.len() < 64 {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    let valid_record = binding[..16] != [0; 16] && binding[16..24] != [0; 8];
    let valid_root = u64::from_le_bytes(binding[56..64].try_into().expect("fixed root")) != 0;
    if !valid_record || !valid_root {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    if tag == 6 {
        let mut extra =
            crate::durability_admission::independent_wal_oracle::ByteCursor::new(&binding[64..]);
        match extra.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
            [0] => {}
            [1] => {
                extra.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 24 + 8 + 32)?;
            }
            _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
        }
        match extra.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
            [0 | 1] => {}
            [2] => {
                extra.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 24)?;
            }
            _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
        }
        extra.finish()?;
        match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
            [0] => {}
            [1] => {
                match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
                    [0] => {}
                    [1] => {
                        cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 24)?;
                        match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
                            [0] => {}
                            [1] => {
                                cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 24 + 8 + 32)?;
                            }
                            _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
                        }
                    }
                    _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
                }
                read_sequence(cursor)?;
            }
            _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
        }
    } else if binding.len() != 64 {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    } else if tag == 5 {
        match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
            [0] => {}
            [1] if !cursor.field(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload)?.is_empty() => {}
            _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
        }
        inspect_directory_replacement(cursor)?;
    }
    Ok(())
}

/// A released drop may carry its atomic directory replacement: the expected
/// previous binding, its payload digest, and the next directory binding.
pub fn inspect_directory_replacement(
    cursor: &mut ByteCursor<'_>,
) -> Result<(), BindingInspectionDenial> {
    match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
        [0] => return Ok(()),
        [1] => {}
        _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
    }
    if cursor.take(
        crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
        1,
    )? != [1]
    {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    cursor.take(
        crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
        24,
    )?; // previous directory record
    inspect_publication(cursor)?;
    cursor.take(
        crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
        32,
    )?; // previous payload digest
    cursor.take(
        crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload,
        64,
    )?; // next directory binding
    inspect_publication(cursor)?;
    match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
        [0 | 1] => Ok(()),
        [2] => cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 24).map(|_| ()),
        _ => Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
    }
}

pub fn inspect_publication(cursor: &mut ByteCursor<'_>) -> Result<(), BindingInspectionDenial> {
    match cursor.take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 1)? {
        [0] => Ok(()),
        [1] => cursor
            .take(crate::durability_admission::independent_wal_oracle::BindingField::RedoPayload, 24 + 8 + 32)
            .map(|_| ()),
        _ => Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
    }
}

pub fn write_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}
