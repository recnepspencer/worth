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

pub(super) fn read_sequence<T, S: PhysicalRecoveryDecodeStorage>(
    cursor: &mut Cursor<'_>,
    maximum: u64,
    storage: &mut S,
    mut read: impl FnMut(&[u8], &mut S) -> Result<T, PhysicalRecoveryDecodeFailure<S::Denial>>,
) -> Result<Vec<T>, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let count = cursor.u64()?;
    if count > maximum {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit.into());
    }
    cursor.require_sequence_backing(count)?;
    let mut values = decode_storage::reserve_vec(
        usize::try_from(count).map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
        storage,
    )?;
    for _ in 0..count {
        values.push(read(cursor.field()?, storage)?);
    }
    Ok(values)
}

#[cfg(test)]
mod count_admission_tests {
    use super::*;
    #[test]
    fn truncated_in_policy_sequences_deny_before_allocation() {
        struct Admission(usize);
        impl PhysicalRecoveryDecodeStorage for Admission {
            type Denial = ();
            fn admit_allocation(&mut self, _: u64) -> Result<(), ()> {
                self.0 += 1;
                Ok(())
            }
        }
        let count_only = 2_u64.to_le_bytes();
        let mut storage = Admission(0);
        let read = |_: &[u8], _: &mut Admission| Ok::<u64, PhysicalRecoveryDecodeFailure<()>>(0);
        let regular = read_sequence(&mut Cursor::new(&count_only), 2, &mut storage, read);
        let bounded =
            read_bounded_sequence(&mut Cursor::new(&count_only), 2, &mut 2, &mut storage, read);
        for outcome in [regular, bounded] {
            assert!(matches!(
                outcome,
                Err(PhysicalRecoveryDecodeFailure::Projection(
                    PhysicalRecoveryProjectionDenial::Malformed
                ))
            ));
        }
        assert_eq!(storage.0, 0);
    }
}

pub(super) fn read_bounded_sequence<T, S: PhysicalRecoveryDecodeStorage>(
    cursor: &mut Cursor<'_>,
    maximum: u64,
    remaining_total: &mut u64,
    storage: &mut S,
    mut read: impl FnMut(&[u8], &mut S) -> Result<T, PhysicalRecoveryDecodeFailure<S::Denial>>,
) -> Result<Vec<T>, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let count = cursor.u64()?;
    if count > maximum || count > *remaining_total {
        return Err(PhysicalRecoveryProjectionDenial::EntryLimit.into());
    }
    cursor.require_sequence_backing(count)?;
    *remaining_total -= count;
    let mut values = decode_storage::reserve_vec(
        usize::try_from(count).map_err(|_| PhysicalRecoveryProjectionDenial::EntryLimit)?,
        storage,
    )?;
    for _ in 0..count {
        values.push(read(cursor.field()?, storage)?);
    }
    Ok(values)
}
