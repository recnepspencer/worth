//! A derived or explicitly overridden identity, held inline so const
//! evaluation can produce it.
use super::grammar::MAXIMUM_APPLICATION_IDENTITY_BYTES;

/// One validated identity of a namespace. Its text is printable ASCII.
#[derive(Clone, Copy)]
pub struct ApplicationDerivedIdentity {
    bytes: [u8; MAXIMUM_APPLICATION_IDENTITY_BYTES],
    length: usize,
}

impl ApplicationDerivedIdentity {
    /// Joins already-validated segments with `.`; the caller checked the bound.
    pub(super) const fn joined(segments: &[&str]) -> Self {
        let mut bytes = [0_u8; MAXIMUM_APPLICATION_IDENTITY_BYTES];
        let mut length = 0;
        let mut segment = 0;
        while segment < segments.len() {
            if segment > 0 {
                bytes[length] = b'.';
                length += 1;
            }
            let text = segments[segment].as_bytes();
            let mut index = 0;
            while index < text.len() {
                bytes[length] = text[index];
                length += 1;
                index += 1;
            }
            segment += 1;
        }
        Self { bytes, length }
    }

    pub const fn len(&self) -> usize {
        self.length
    }

    /// Always false: every namespace identity has at least one segment.
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.bytes.split_at(self.length).0) {
            Ok(text) => text,
            // Unreachable: construction admits printable ASCII only.
            Err(_) => "",
        }
    }

    /// Copies the identity into an exact-length array for
    /// `worth_query_identity!`, which turns it into a `&'static str`.
    #[doc(hidden)]
    pub const fn to_bytes<const LENGTH: usize>(&self) -> [u8; LENGTH] {
        let mut bytes = [0_u8; LENGTH];
        let mut index = 0;
        while index < LENGTH && index < self.length {
            bytes[index] = self.bytes[index];
            index += 1;
        }
        bytes
    }
}

impl PartialEq for ApplicationDerivedIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for ApplicationDerivedIdentity {}

impl std::hash::Hash for ApplicationDerivedIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl std::fmt::Debug for ApplicationDerivedIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("ApplicationDerivedIdentity")
            .field(&self.as_str())
            .finish()
    }
}

impl std::fmt::Display for ApplicationDerivedIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
