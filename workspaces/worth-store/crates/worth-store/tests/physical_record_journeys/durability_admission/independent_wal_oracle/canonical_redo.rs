use super::{BindingField, BindingInspectionDenial, ByteCursor, IndependentRedoTargetClaim};

const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
const CURRENT_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v16";

pub(super) fn independent_canonical_redo(
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

pub(super) fn independent_recovery_projection(
    canonical_redo: &[u8],
) -> Result<&[u8], BindingInspectionDenial> {
    let mut cursor = ByteCursor::new(canonical_redo);
    if cursor.field(BindingField::RedoPayload)? != REDO_DOMAIN {
        return Err(BindingInspectionDenial::DomainMismatch);
    }
    let record_count = cursor.read_u64()?;
    if record_count == 0 {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    for _ in 0..record_count {
        cursor.take(BindingField::RedoPayload, 4)?;
        cursor.take(BindingField::RedoPayload, 8)?;
        let target_count = cursor.read_u64()?;
        if target_count == 0 {
            return Err(BindingInspectionDenial::InvalidFrame);
        }
        for _ in 0..target_count {
            cursor.field(BindingField::RedoPayload)?;
            cursor.take(BindingField::RedoPayload, 32)?;
        }
        cursor.field(BindingField::RedoPayload)?;
    }
    let projection = cursor.field(BindingField::RedoPayload)?;
    cursor.finish()?;

    inspect_recovery_projection(projection)?;
    Ok(projection)
}

fn inspect_recovery_projection(projection: &[u8]) -> Result<(), BindingInspectionDenial> {
    let mut cursor = ByteCursor::new(projection);
    let domain = cursor.field(BindingField::RedoPayload)?;
    if domain != CURRENT_PROJECTION_DOMAIN {
        return Err(BindingInspectionDenial::DomainMismatch);
    }
    if cursor.read_u64()? == 0 || cursor.field(BindingField::RedoPayload)?.is_empty() {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    read_sequence(&mut cursor)?; // selected record identities
    let source_copy = match cursor.take(BindingField::RedoPayload, 1)? {
        [0] => {
            read_sequence(&mut cursor)?; // physical frames
            false
        }
        [1] => {
            cursor.field(BindingField::RedoPayload)?; // embedded copy intent
            cursor.read_u64()?; // intent LSN
            cursor.take(BindingField::RedoPayload, 32)?; // intent digest
            true
        }
        _ => return Err(BindingInspectionDenial::InvalidFrame),
    };
    read_sequence(&mut cursor)?; // placements
    read_sequence(&mut cursor)?; // segment updates
    read_sequence(&mut cursor)?; // manifests
    inspect_operation(&mut cursor, source_copy)?;
    cursor.finish()
}

fn read_sequence(cursor: &mut ByteCursor<'_>) -> Result<(), BindingInspectionDenial> {
    let count = cursor.read_u64()?;
    for _ in 0..count {
        cursor.field(BindingField::RedoPayload)?;
    }
    Ok(())
}

fn inspect_operation(
    cursor: &mut ByteCursor<'_>,
    source_copy: bool,
) -> Result<(), BindingInspectionDenial> {
    let bytes = cursor.field(BindingField::RedoPayload)?;
    let Some((&tag, binding)) = bytes.split_first() else {
        return Err(BindingInspectionDenial::InvalidFrame);
    };
    if tag == 0 {
        return (binding.is_empty())
            .then_some(())
            .ok_or(BindingInspectionDenial::InvalidFrame);
    }
    if source_copy || !matches!(tag, 1..=8) || binding.len() < 64 {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    let valid_record = binding[..16] != [0; 16] && binding[16..24] != [0; 8];
    let valid_root = u64::from_le_bytes(binding[56..64].try_into().expect("fixed root")) != 0;
    if !valid_record || !valid_root {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    if tag == 6 {
        let mut extra = ByteCursor::new(&binding[64..]);
        match extra.take(BindingField::RedoPayload, 1)? {
            [0] => {}
            [1] => {
                extra.take(BindingField::RedoPayload, 24 + 8 + 32)?;
            }
            _ => return Err(BindingInspectionDenial::InvalidFrame),
        }
        match extra.take(BindingField::RedoPayload, 1)? {
            [0 | 1] => {}
            [2] => {
                extra.take(BindingField::RedoPayload, 24)?;
            }
            _ => return Err(BindingInspectionDenial::InvalidFrame),
        }
        extra.finish()?;
        match cursor.take(BindingField::RedoPayload, 1)? {
            [0] => {}
            [1] => {
                match cursor.take(BindingField::RedoPayload, 1)? {
                    [0] => {}
                    [1] => {
                        cursor.take(BindingField::RedoPayload, 24)?;
                        match cursor.take(BindingField::RedoPayload, 1)? {
                            [0] => {}
                            [1] => {
                                cursor.take(BindingField::RedoPayload, 24 + 8 + 32)?;
                            }
                            _ => return Err(BindingInspectionDenial::InvalidFrame),
                        }
                    }
                    _ => return Err(BindingInspectionDenial::InvalidFrame),
                }
                read_sequence(cursor)?;
            }
            _ => return Err(BindingInspectionDenial::InvalidFrame),
        }
    } else if binding.len() != 64 {
        return Err(BindingInspectionDenial::InvalidFrame);
    } else if tag == 5 {
        match cursor.take(BindingField::RedoPayload, 1)? {
            [0] => {}
            [1] if !cursor.field(BindingField::RedoPayload)?.is_empty() => {}
            _ => return Err(BindingInspectionDenial::InvalidFrame),
        }
        inspect_directory_replacement(cursor)?;
    }
    Ok(())
}

/// A released drop may carry its atomic directory replacement: the expected
/// previous binding, its payload digest, and the next directory binding.
fn inspect_directory_replacement(
    cursor: &mut ByteCursor<'_>,
) -> Result<(), BindingInspectionDenial> {
    match cursor.take(BindingField::RedoPayload, 1)? {
        [0] => return Ok(()),
        [1] => {}
        _ => return Err(BindingInspectionDenial::InvalidFrame),
    }
    if cursor.take(BindingField::RedoPayload, 1)? != [1] {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    cursor.take(BindingField::RedoPayload, 24)?; // previous directory record
    inspect_publication(cursor)?;
    cursor.take(BindingField::RedoPayload, 32)?; // previous payload digest
    cursor.take(BindingField::RedoPayload, 64)?; // next directory binding
    inspect_publication(cursor)?;
    match cursor.take(BindingField::RedoPayload, 1)? {
        [0 | 1] => Ok(()),
        [2] => cursor.take(BindingField::RedoPayload, 24).map(|_| ()),
        _ => Err(BindingInspectionDenial::InvalidFrame),
    }
}

fn inspect_publication(cursor: &mut ByteCursor<'_>) -> Result<(), BindingInspectionDenial> {
    match cursor.take(BindingField::RedoPayload, 1)? {
        [0] => Ok(()),
        [1] => cursor
            .take(BindingField::RedoPayload, 24 + 8 + 32)
            .map(|_| ()),
        _ => Err(BindingInspectionDenial::InvalidFrame),
    }
}

fn write_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}

#[cfg(test)]
mod tests {
    use super::{inspect_recovery_projection, write_field, CURRENT_PROJECTION_DOMAIN};

    fn projection(domain: &[u8], source_copy: bool, operation: Option<&[u8]>) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_field(&mut bytes, domain);
        bytes.extend_from_slice(&4_u64.to_le_bytes());
        write_field(&mut bytes, b"root");
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        write_field(&mut bytes, &[7; 24]);
        if source_copy {
            bytes.push(1);
            write_field(&mut bytes, b"copy-intent");
            bytes.extend_from_slice(&3_u64.to_le_bytes());
            bytes.extend_from_slice(&[9; 32]);
        } else {
            bytes.push(0);
            bytes.extend_from_slice(&1_u64.to_le_bytes());
            write_field(&mut bytes, b"frame");
        }
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        write_field(&mut bytes, b"placement");
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        if let Some(operation) = operation {
            write_field(&mut bytes, operation);
        }
        bytes
    }

    #[test]
    fn projection_oracle_admits_current_domain_and_both_physical_variants() {
        for source_copy in [false, true] {
            assert!(inspect_recovery_projection(&projection(
                CURRENT_PROJECTION_DOMAIN,
                source_copy,
                Some(&[0])
            ))
            .is_ok());
        }
    }

    #[test]
    fn projection_oracle_denies_missing_or_malformed_current_operation() {
        for semantic in [None, Some(&[][..]), Some(&[0, 0][..]), Some(&[3][..])] {
            assert!(inspect_recovery_projection(&projection(
                CURRENT_PROJECTION_DOMAIN,
                false,
                semantic
            ))
            .is_err());
        }
        assert!(inspect_recovery_projection(&projection(
            b"store.physical.recovery-projection.v6",
            false,
            Some(&[0])
        ))
        .is_err());
        let mut bound = vec![1];
        bound.extend_from_slice(&[7; 24]);
        bound.extend_from_slice(&[8; 32]);
        bound.extend_from_slice(&5_u64.to_le_bytes());
        assert!(inspect_recovery_projection(&projection(
            CURRENT_PROJECTION_DOMAIN,
            false,
            Some(&bound)
        ))
        .is_ok());
        assert!(inspect_recovery_projection(&projection(
            CURRENT_PROJECTION_DOMAIN,
            true,
            Some(&bound)
        ))
        .is_err());
    }
}
