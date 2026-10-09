//! One field encoding grammar can produce bytes or compare canonical input.

pub(in crate::physical_runtime) trait CanonicalBindingEncoding {
    fn write(&mut self, bytes: &[u8]);

    fn push(&mut self, byte: u8) {
        self.write(&[byte]);
    }

    fn field(&mut self, bytes: &[u8]) {
        self.write(&(bytes.len() as u64).to_le_bytes());
        self.write(bytes);
    }
}

impl CanonicalBindingEncoding for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

pub(super) struct CanonicalBindingComparison<'bytes> {
    remaining: &'bytes [u8],
    equal: bool,
}

impl<'bytes> CanonicalBindingComparison<'bytes> {
    pub(super) fn new(bytes: &'bytes [u8]) -> Self {
        Self {
            remaining: bytes,
            equal: true,
        }
    }

    pub(super) fn matches(self) -> bool {
        self.equal && self.remaining.is_empty()
    }
}

impl CanonicalBindingEncoding for CanonicalBindingComparison<'_> {
    fn write(&mut self, bytes: &[u8]) {
        let Some((observed, remaining)) = self.remaining.split_at_checked(bytes.len()) else {
            self.equal = false;
            self.remaining = &[];
            return;
        };
        self.equal &= observed == bytes;
        self.remaining = remaining;
    }
}

#[cfg(test)]
mod tests;
