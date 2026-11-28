//! Canonical big-endian primitives for naming pages and durable naming frames.

/// Appends canonical primitive encodings.
#[derive(Default)]
pub(crate) struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub(crate) fn raw(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub(crate) fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(crate) fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub(crate) fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub(crate) fn digest(&mut self, digest: &[u8; 32]) {
        self.bytes.extend_from_slice(digest);
    }
}

/// A short read leaves the cursor unchanged so the caller can report its section.
#[derive(Debug)]
pub(crate) struct Truncated {
    pub(crate) needed: usize,
    pub(crate) remaining: usize,
}

pub(crate) struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub(crate) fn remaining(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn take(&mut self, needed: usize) -> Result<&'a [u8], Truncated> {
        let (head, rest) = self.0.split_at_checked(needed).ok_or(Truncated {
            needed,
            remaining: self.remaining(),
        })?;
        self.0 = rest;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], Truncated> {
        Ok(self
            .take(N)?
            .try_into()
            .expect("take returns exactly the requested length"))
    }

    pub(crate) fn u8(&mut self) -> Result<u8, Truncated> {
        Ok(self.take(1)?[0])
    }

    pub(crate) fn u32(&mut self) -> Result<u32, Truncated> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, Truncated> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    pub(crate) fn digest(&mut self) -> Result<[u8; 32], Truncated> {
        self.array()
    }
}
