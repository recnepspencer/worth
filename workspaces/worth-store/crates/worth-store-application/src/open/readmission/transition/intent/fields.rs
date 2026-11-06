//! The intent's closed positional fields, shared by its one decoder.
use super::ApplicationHomeTransitionIntentDenial as Denial;

pub(super) struct Read<'a>(pub(super) &'a [u8]);
impl<'a> Read<'a> {
    pub(super) fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Denial> {
        let (value, remaining) = self.0.split_first_chunk::<N>().ok_or(Denial::Malformed)?;
        self.0 = remaining;
        Ok(*value)
    }
    pub(super) fn u64(&mut self) -> Result<u64, Denial> {
        self.fixed().map(u64::from_le_bytes)
    }
    pub(super) fn u32(&mut self) -> Result<u32, Denial> {
        self.fixed().map(u32::from_le_bytes)
    }
    pub(super) fn bytes(&mut self) -> Result<&'a [u8], Denial> {
        let size = self.u32()? as usize;
        let (value, remaining) = self.0.split_at_checked(size).ok_or(Denial::Malformed)?;
        self.0 = remaining;
        Ok(value)
    }
    pub(super) fn text(&mut self) -> Result<&'a str, Denial> {
        std::str::from_utf8(self.bytes()?).map_err(|_| Denial::Malformed)
    }
}
