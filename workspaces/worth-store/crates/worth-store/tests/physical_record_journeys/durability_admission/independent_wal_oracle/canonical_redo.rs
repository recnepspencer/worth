use super::{BindingField, BindingInspectionDenial, ByteCursor, IndependentRedoTargetClaim};

const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
const V5_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v5";
const V6_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v6";

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
    let v6 = if domain == V6_PROJECTION_DOMAIN {
        true
    } else if domain == V5_PROJECTION_DOMAIN {
        false
    } else {
        return Err(BindingInspectionDenial::DomainMismatch);
    };
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
    if v6 {
        let semantic = cursor.field(BindingField::RedoPayload)?;
        if !valid_blob_semantic(semantic) || (source_copy && semantic != [0]) {
            return Err(BindingInspectionDenial::InvalidFrame);
        }
    }
    read_sequence(&mut cursor)?; // placements
    read_sequence(&mut cursor)?; // segment updates
    read_sequence(&mut cursor)?; // manifests
    cursor.finish()
}

fn read_sequence(cursor: &mut ByteCursor<'_>) -> Result<(), BindingInspectionDenial> {
    let count = cursor.read_u64()?;
    for _ in 0..count {
        cursor.field(BindingField::RedoPayload)?;
    }
    Ok(())
}

fn valid_blob_semantic(bytes: &[u8]) -> bool {
    match bytes {
        [0] => true,
        [1 | 2, rest @ ..] if rest.len() == 64 => {
            rest[..16] != [0; 16]
                && rest[16..24] != [0; 8]
                && u64::from_le_bytes(rest[56..64].try_into().expect("fixed root")) != 0
        }
        _ => false,
    }
}

fn write_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}

#[cfg(test)]
mod tests {
    use super::{
        inspect_recovery_projection, write_field, V5_PROJECTION_DOMAIN, V6_PROJECTION_DOMAIN,
    };

    fn projection(domain: &[u8], source_copy: bool, semantic: Option<&[u8]>) -> Vec<u8> {
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
        if let Some(semantic) = semantic {
            write_field(&mut bytes, semantic);
        }
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        write_field(&mut bytes, b"placement");
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes
    }

    #[test]
    fn projection_oracle_admits_both_versions_and_both_physical_variants() {
        for source_copy in [false, true] {
            assert!(inspect_recovery_projection(&projection(
                V5_PROJECTION_DOMAIN,
                source_copy,
                None
            ))
            .is_ok());
            assert!(inspect_recovery_projection(&projection(
                V6_PROJECTION_DOMAIN,
                source_copy,
                Some(&[0])
            ))
            .is_ok());
        }
    }

    #[test]
    fn projection_oracle_denies_missing_or_malformed_v6_semantic() {
        for semantic in [None, Some(&[][..]), Some(&[0, 0][..]), Some(&[3][..])] {
            assert!(inspect_recovery_projection(&projection(
                V6_PROJECTION_DOMAIN,
                false,
                semantic
            ))
            .is_err());
        }
        assert!(
            inspect_recovery_projection(&projection(V5_PROJECTION_DOMAIN, false, Some(&[0])))
                .is_err()
        );
        let mut bound = vec![1];
        bound.extend_from_slice(&[7; 24]);
        bound.extend_from_slice(&[8; 32]);
        bound.extend_from_slice(&5_u64.to_le_bytes());
        assert!(inspect_recovery_projection(&projection(
            V6_PROJECTION_DOMAIN,
            false,
            Some(&bound)
        ))
        .is_ok());
        assert!(
            inspect_recovery_projection(&projection(V6_PROJECTION_DOMAIN, true, Some(&bound)))
                .is_err()
        );
    }
}
