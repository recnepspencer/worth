/// Opaque native bytes for one complete Relational recovery checkpoint.
///
/// Construction accepts untrusted bytes deliberately: admission belongs to
/// the recovery authority, where codec integrity and runtime continuity are
/// checked together. Callers can retain or transport the bytes, but cannot
/// inspect or selectively rewrite Relational truth through this type.
pub struct RelationalNativeCheckpoint {
    bytes: Box<[u8]>,
    region: std::ops::Range<usize>,
}

impl Clone for RelationalNativeCheckpoint {
    fn clone(&self) -> Self {
        Self::from_untrusted_bytes(self.bytes().to_vec())
    }
}

impl std::fmt::Debug for RelationalNativeCheckpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalNativeCheckpoint")
            .field("byte_len", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for RelationalNativeCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}

impl Eq for RelationalNativeCheckpoint {}

impl RelationalNativeCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        let bytes = bytes.into();
        let region = 0..bytes.len();
        Self { bytes, region }
    }

    /// Retain an embedded native payload without copying its enclosing buffer.
    /// Recovery still authenticates and readmits the selected native bytes.
    pub fn from_untrusted_bytes_region(
        bytes: Box<[u8]>,
        region: std::ops::Range<usize>,
    ) -> Result<Self, &'static str> {
        if bytes.get(region.clone()).is_none() {
            return Err("native checkpoint byte region is out of bounds");
        }
        Ok(Self { bytes, region })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }

    pub(crate) fn from_captured_bytes(bytes: Vec<u8>) -> Self {
        let region = 0..bytes.len();
        Self {
            bytes: bytes.into_boxed_slice(),
            region,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RelationalNativeCheckpoint;

    #[test]
    fn embedded_native_payload_retains_its_owned_buffer_without_copying() {
        let bytes = b"prefixnative-suffix".to_vec().into_boxed_slice();
        let expected = bytes.as_ptr().wrapping_add(6);
        let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes_region(bytes, 6..12)
            .expect("the bounded native region is admitted");
        assert_eq!(checkpoint.bytes(), b"native");
        assert_eq!(checkpoint.bytes().as_ptr(), expected);
        assert_eq!(
            checkpoint,
            RelationalNativeCheckpoint::from_untrusted_bytes(b"native".to_vec())
        );
        let cloned = checkpoint.clone();
        assert_eq!(cloned.bytes(), b"native");
        assert_eq!(cloned.region, 0..6);
    }

    #[test]
    fn embedded_native_payload_rejects_invalid_range() {
        let bytes = b"native".to_vec().into_boxed_slice();
        assert!(RelationalNativeCheckpoint::from_untrusted_bytes_region(bytes, 3..7).is_err());
    }
}
