/// Opaque native bytes for one complete Relational recovery checkpoint.
///
/// Construction accepts untrusted bytes deliberately: admission belongs to
/// the recovery authority, where codec integrity and runtime continuity are
/// checked together. Callers can retain or transport the bytes, but cannot
/// inspect or selectively rewrite Relational truth through this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalNativeCheckpoint {
    bytes: Box<[u8]>,
}

impl RelationalNativeCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn from_captured_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes: bytes.into_boxed_slice(),
        }
    }
}
