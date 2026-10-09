use super::*;

pub(super) fn write_record(target: &mut Vec<u8>, record: PersistedRecordIdentity) {
    target.extend_from_slice(&record.allocation_epoch());
    target.extend_from_slice(&record.ordinal().to_le_bytes());
}
pub(super) fn read_record(
    cursor: &mut Cursor<'_>,
) -> Result<PersistedRecordIdentity, PhysicalRecoveryProjectionDenial> {
    PersistedRecordIdentity::new(cursor.array()?, cursor.u64()?)
        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)
}
pub(super) fn generation(
    value: u64,
) -> Result<PhysicalGeneration, PhysicalRecoveryProjectionDenial> {
    PhysicalGeneration::from_raw(value).map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)
}
pub(super) fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}

pub(super) struct Cursor<'a> {
    remaining: &'a [u8],
}
impl<'a> Cursor<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }
    pub(super) fn require_sequence_backing(
        &self,
        count: u64,
    ) -> Result<(), PhysicalRecoveryProjectionDenial> {
        let required = usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(8));
        if required.is_none_or(|required| required > self.remaining.len()) {
            return Err(PhysicalRecoveryProjectionDenial::Malformed);
        }
        Ok(())
    }
    pub(super) fn take(
        &mut self,
        len: usize,
    ) -> Result<&'a [u8], PhysicalRecoveryProjectionDenial> {
        let (head, tail) = self
            .remaining
            .split_at_checked(len)
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
        self.remaining = tail;
        Ok(head)
    }
    pub(super) fn byte(&mut self) -> Result<u8, PhysicalRecoveryProjectionDenial> {
        Ok(self.take(1)?[0])
    }
    pub(super) fn u16(&mut self) -> Result<u16, PhysicalRecoveryProjectionDenial> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub(super) fn u32(&mut self) -> Result<u32, PhysicalRecoveryProjectionDenial> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub(super) fn u64(&mut self) -> Result<u64, PhysicalRecoveryProjectionDenial> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], PhysicalRecoveryProjectionDenial> {
        self.take(N)?
            .try_into()
            .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)
    }
    pub(super) fn field(&mut self) -> Result<&'a [u8], PhysicalRecoveryProjectionDenial> {
        let len = usize::try_from(self.u64()?)
            .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
        self.take(len)
    }
    pub(super) fn end(self) -> Result<(), PhysicalRecoveryProjectionDenial> {
        self.remaining
            .is_empty()
            .then_some(())
            .ok_or(PhysicalRecoveryProjectionDenial::Malformed)
    }
}
