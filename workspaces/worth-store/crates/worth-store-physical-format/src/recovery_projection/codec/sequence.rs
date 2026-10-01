use super::*;

pub(super) fn write_sequence<T>(
    target: &mut Vec<u8>,
    values: &[T],
    mut write: impl FnMut(&mut Vec<u8>, &T),
) {
    target.extend_from_slice(&(values.len() as u64).to_le_bytes());
    for value in values {
        let mut encoded = Vec::new();
        write(&mut encoded, value);
        field(target, &encoded);
    }
}

pub(super) fn read_sequence<T>(
    cursor: &mut Cursor<'_>,
    maximum: u64,
    read: fn(&[u8]) -> Result<T, PhysicalRecoveryProjectionDenial>,
) -> Result<Vec<T>, PhysicalRecoveryProjectionDenial> {
    let count = cursor.u64()?;
    if count > maximum {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit);
    }
    let mut values = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(read(cursor.field()?)?);
    }
    Ok(values)
}

pub(super) fn read_bounded_sequence<T>(
    cursor: &mut Cursor<'_>,
    maximum: u64,
    remaining_total: &mut u64,
    mut read: impl FnMut(&[u8]) -> Result<T, PhysicalRecoveryProjectionDenial>,
) -> Result<Vec<T>, PhysicalRecoveryProjectionDenial> {
    let count = cursor.u64()?;
    if count > maximum || count > *remaining_total {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit);
    }
    *remaining_total -= count;
    let mut values = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(read(cursor.field()?)?);
    }
    Ok(values)
}
