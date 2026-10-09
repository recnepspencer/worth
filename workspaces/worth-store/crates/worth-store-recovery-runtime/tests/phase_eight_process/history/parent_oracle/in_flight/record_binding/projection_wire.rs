use super::RecordIdentity;

pub(super) struct Cursor<'bytes> {
    bytes: &'bytes [u8],
    offset: usize,
}

impl<'bytes> Cursor<'bytes> {
    pub(super) const fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(super) fn field(&mut self) -> Result<&'bytes [u8], String> {
        let length = usize::try_from(self.u64()?).map_err(|_| "field length overflowed")?;
        let end = self
            .offset
            .checked_add(length)
            .ok_or("field length overflowed")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("field is truncated")?;
        self.offset = end;
        Ok(value)
    }

    pub(super) fn record(&mut self) -> Result<RecordIdentity, String> {
        let bytes = self.field()?;
        Self::decode_record(bytes)
    }

    pub(super) fn raw_record(&mut self) -> Result<RecordIdentity, String> {
        Self::decode_record(self.take(24)?)
    }

    fn decode_record(bytes: &[u8]) -> Result<RecordIdentity, String> {
        if bytes.len() != 24 {
            return Err("record identity field has the wrong width".to_owned());
        }
        Ok(RecordIdentity {
            allocation_epoch: bytes[..16]
                .try_into()
                .map_err(|_| "record epoch is truncated")?,
            ordinal: u64::from_le_bytes(
                bytes[16..24]
                    .try_into()
                    .map_err(|_| "record ordinal is truncated")?,
            ),
        })
    }

    pub(super) fn u16(&mut self) -> Result<u16, String> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes(
            bytes.try_into().map_err(|_| "u16 is truncated")?,
        ))
    }

    pub(super) fn u32(&mut self) -> Result<u32, String> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(
            bytes.try_into().map_err(|_| "u32 is truncated")?,
        ))
    }

    pub(super) fn u64(&mut self) -> Result<u64, String> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes(
            bytes.try_into().map_err(|_| "u64 is truncated")?,
        ))
    }

    pub(super) fn byte(&mut self) -> Result<u8, String> {
        Ok(*self.take(1)?.first().ok_or("byte is truncated")?)
    }

    pub(super) fn take(&mut self, length: usize) -> Result<&'bytes [u8], String> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or("cursor offset overflowed")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("cursor is truncated")?;
        self.offset = end;
        Ok(value)
    }

    pub(super) const fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

/// V13's final seven placement bytes are route class, tier, and reserved
/// metadata. This is the parent observer's independent wire check, not a
/// production route decoder or a grant of selected-control authority.
pub(super) fn classified_route_metadata(bytes: &[u8]) -> bool {
    if bytes.len() != 7 || bytes[5..] != [0; 2] || bytes[4] > 2 {
        return false;
    }
    let family = u16::from_le_bytes([bytes[2], bytes[3]]);
    match (bytes[0], bytes[1], family) {
        (0, 0, 0) => bytes[4] == 0,
        (1, 0, 0) | (4, 0, 0) => true,
        (2, 1..=16, 0) => true,
        (3, 0, 1..) => true,
        _ => false,
    }
}
