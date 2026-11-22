//! Big-endian primitive encoding of the durable persistent-name log.
//!
//! Lengths and counts are 32-bit; text is length-prefixed UTF-8; identities
//! are raw 32-byte digests. The reader names the section it reads so a
//! truncation reports where the log was torn.

use super::refusal::{PersistentNameDurabilityError as Refusal, PersistentNameWireSection};

/// Appends canonical primitive encodings.
#[derive(Default)]
pub(super) struct WireWriter {
    bytes: Vec<u8>,
}

impl WireWriter {
    pub(super) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub(super) fn raw(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub(super) fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub(super) fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub(super) fn digest(&mut self, digest: &[u8; 32]) {
        self.bytes.extend_from_slice(digest);
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

    pub(super) fn option_tag(&mut self, present: bool) {
        self.u8(u8::from(present));
    }
}

/// Reads canonical primitive encodings, refusing truncation typed.
pub(super) struct WireReader<'bytes> {
    bytes: &'bytes [u8],
    section: PersistentNameWireSection,
}

impl<'bytes> WireReader<'bytes> {
    pub(super) const fn new(bytes: &'bytes [u8], section: PersistentNameWireSection) -> Self {
        Self { bytes, section }
    }

    pub(super) const fn remaining(&self) -> &'bytes [u8] {
        self.bytes
    }

    pub(super) fn take(&mut self, needed: usize) -> Result<&'bytes [u8], Refusal> {
        if self.bytes.len() < needed {
            return Err(Refusal::Truncated {
                section: self.section,
                needed,
                remaining: self.bytes.len(),
            });
        }
        let (head, rest) = self.bytes.split_at(needed);
        self.bytes = rest;
        Ok(head)
    }

    pub(super) fn u8(&mut self) -> Result<u8, Refusal> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn u32(&mut self) -> Result<u32, Refusal> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes(
            bytes.try_into().expect("take returned four bytes"),
        ))
    }

    pub(super) fn u64(&mut self) -> Result<u64, Refusal> {
        let bytes = self.take(8)?;
        Ok(u64::from_be_bytes(
            bytes.try_into().expect("take returned eight bytes"),
        ))
    }

    pub(super) fn digest(&mut self) -> Result<[u8; 32], Refusal> {
        let bytes = self.take(32)?;
        Ok(bytes.try_into().expect("take returned thirty-two bytes"))
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

    /// Reads a canonical presence tag: `0` absent, `1` present.
    pub(super) fn option_tag(&mut self, field: &'static str) -> Result<bool, Refusal> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            tag => Err(Refusal::InvalidTag { field, tag }),
        }
    }

    /// Requires the section to be fully consumed.
    pub(super) fn finish(self) -> Result<(), Refusal> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(Refusal::TrailingBytes {
                remaining: self.bytes.len(),
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
