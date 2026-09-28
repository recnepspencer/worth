use super::{BindingField, BindingInspectionDenial, ByteCursor, IndependentRedoTargetClaim};

const REDO_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
const PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v5";

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

    let mut projection_cursor = ByteCursor::new(projection);
    if projection_cursor.field(BindingField::RedoPayload)? != PROJECTION_DOMAIN
        || projection_cursor.read_u64()? == 0
    {
        return Err(BindingInspectionDenial::DomainMismatch);
    }
    if projection_cursor
        .field(BindingField::RedoPayload)?
        .is_empty()
    {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    Ok(projection)
}

fn write_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}
