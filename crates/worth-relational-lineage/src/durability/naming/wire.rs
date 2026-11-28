//! Big-endian primitive encoding of the durable persistent-name log.
//!
//! Lengths and counts are 32-bit; text is length-prefixed UTF-8; identities
//! are raw 32-byte digests. The reader names the section it reads so a
//! truncation reports where the log was torn.

use super::refusal::{PersistentNameDurabilityError as Refusal, PersistentNameWireSection};

use crate::naming_wire::{Reader, Writer};

#[derive(Default)]
pub(super) struct WireWriter(Writer);

impl WireWriter {
    pub(super) fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    pub(super) fn raw(&mut self, value: &[u8]) {
        self.0.raw(value);
    }

    pub(super) fn u8(&mut self, value: u8) {
        self.0.u8(value);
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.0.u32(value);
    }

    pub(super) fn u64(&mut self, value: u64) {
        self.0.u64(value);
    }

    pub(super) fn digest(&mut self, value: &[u8; 32]) {
        self.0.digest(value);
    }

    /// Writes a 32-bit count, refusing counts the format cannot carry.
    pub(super) fn count(&mut self, field: &'static str, len: usize) -> Result<(), Refusal> {
        let count = u32::try_from(len).map_err(|_| Refusal::OversizedField { field, len })?;
        self.u32(count);
        Ok(())
    }

    pub(super) fn bytes(&mut self, field: &'static str, bytes: &[u8]) -> Result<(), Refusal> {
        self.count(field, bytes.len())?;
        self.raw(bytes);
        Ok(())
    }

    pub(super) fn text(&mut self, field: &'static str, text: &str) -> Result<(), Refusal> {
        self.bytes(field, text.as_bytes())
    }
}

/// Reads canonical primitive encodings, refusing truncation typed.
pub(super) struct WireReader<'bytes> {
    reader: Reader<'bytes>,
    section: PersistentNameWireSection,
}

impl<'bytes> WireReader<'bytes> {
    pub(super) const fn new(bytes: &'bytes [u8], section: PersistentNameWireSection) -> Self {
        Self { reader: Reader::new(bytes), section }
    }

    fn truncated(&self, error: crate::naming_wire::Truncated) -> Refusal {
        Refusal::Truncated {
            section: self.section,
            needed: error.needed,
            remaining: error.remaining,
        }
    }

    pub(super) fn take(&mut self, needed: usize) -> Result<&'bytes [u8], Refusal> {
        self.reader.take(needed).map_err(|error| self.truncated(error))
    }

    pub(super) fn u8(&mut self) -> Result<u8, Refusal> {
        self.reader.u8().map_err(|error| self.truncated(error))
    }

    pub(super) fn u32(&mut self) -> Result<u32, Refusal> {
        self.reader.u32().map_err(|error| self.truncated(error))
    }

    pub(super) fn u64(&mut self) -> Result<u64, Refusal> {
        self.reader.u64().map_err(|error| self.truncated(error))
    }

    pub(super) fn digest(&mut self) -> Result<[u8; 32], Refusal> {
        self.reader.digest().map_err(|error| self.truncated(error))
    }

    pub(super) fn count(&mut self) -> Result<usize, Refusal> {
        Ok(usize::try_from(self.u32()?).expect("u32 count fits usize"))
    }

    pub(super) fn bytes(&mut self) -> Result<Vec<u8>, Refusal> {
        let len = self.count()?;
        Ok(self.take(len)?.to_vec())
    }

    pub(super) fn text(&mut self, field: &'static str) -> Result<String, Refusal> {
        String::from_utf8(self.bytes()?).map_err(|_| Refusal::MalformedUtf8 { field })
    }

    /// Requires the section to be fully consumed.
    pub(super) fn finish(self) -> Result<(), Refusal> {
        if self.reader.remaining() == 0 {
            Ok(())
        } else {
            Err(Refusal::TrailingBytes {
                remaining: self.reader.remaining(),
            })
        }
    }
}

/// Reads one element per count, requiring strictly ascending `key` order so
/// no key repeats and no two encodings of one set or map exist.
pub(super) fn read_strictly_ascending<T, K: Ord>(
    reader: &mut WireReader<'_>,
    field: &'static str,
    mut read: impl FnMut(&mut WireReader<'_>) -> Result<T, Refusal>,
    key: impl Fn(&T) -> K,
) -> Result<Vec<T>, Refusal> {
    let count = reader.count()?;
    let mut items: Vec<T> = Vec::new();
    for _ in 0..count {
        let item = read(reader)?;
        if items
            .last()
            .is_some_and(|previous| key(previous) >= key(&item))
        {
            return Err(Refusal::NonCanonical { field });
        }
        items.push(item);
    }
    Ok(items)
}
