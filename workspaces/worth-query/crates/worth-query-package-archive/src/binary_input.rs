use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

pub(crate) struct BinaryInput<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BinaryInput<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N], Denial> {
        self.take(N)?
            .try_into()
            .map_err(|_| Denial::new(Kind::Truncated))
    }

    pub(crate) fn u16(&mut self) -> Result<u16, Denial> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    pub(crate) fn u8(&mut self) -> Result<u8, Denial> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u32(&mut self) -> Result<u32, Denial> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, Denial> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    pub(crate) fn i8(&mut self) -> Result<i8, Denial> {
        Ok(i8::from_be_bytes(self.array()?))
    }

    pub(crate) fn i16(&mut self) -> Result<i16, Denial> {
        Ok(i16::from_be_bytes(self.array()?))
    }

    pub(crate) fn i32(&mut self) -> Result<i32, Denial> {
        Ok(i32::from_be_bytes(self.array()?))
    }

    pub(crate) fn i64(&mut self) -> Result<i64, Denial> {
        Ok(i64::from_be_bytes(self.array()?))
    }

    pub(crate) fn text(&mut self) -> Result<&'a str, Denial> {
        let length = usize::try_from(self.u32()?).map_err(|_| Denial::new(Kind::Truncated))?;
        std::str::from_utf8(self.take(length)?).map_err(|_| Denial::new(Kind::InvalidUtf8))
    }

    pub(crate) fn take(&mut self, length: usize) -> Result<&'a [u8], Denial> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| Denial::new(Kind::Truncated))?;
        let source = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| Denial::new(Kind::Truncated))?;
        self.offset = end;
        Ok(source)
    }

    pub(crate) const fn remaining_len(&self) -> usize {
        self.bytes.len() - self.offset
    }

    pub(crate) fn remaining_starts_with(&self, prefix: &[u8]) -> bool {
        self.bytes[self.offset..].starts_with(prefix)
    }

    pub(crate) const fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
