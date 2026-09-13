/// Immutable bounded bytes presented for physical-integrity validation.
///
/// Construction is intentionally public: this value describes untrusted input
/// and grants no media, resident-frame, or decoder authority.
#[derive(Debug, Clone, Copy)]
pub struct UntrustedPhysicalArtifact<'media> {
    bytes: &'media [u8],
}

impl<'media> UntrustedPhysicalArtifact<'media> {
    pub const fn from_bounded_bytes(bytes: &'media [u8]) -> Self {
        Self { bytes }
    }

    pub const fn bytes(self) -> &'media [u8] {
        self.bytes
    }

    pub const fn byte_count(self) -> u64 {
        self.bytes.len() as u64
    }

    pub(crate) fn same_incarnation(self, other: Self) -> bool {
        self.bytes.len() == other.bytes.len()
            && core::ptr::eq(self.bytes.as_ptr(), other.bytes.as_ptr())
    }
}

#[cfg(test)]
mod tests {
    use super::UntrustedPhysicalArtifact;

    #[test]
    fn construction_describes_bytes_without_claiming_validity() {
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(b"not-yet-valid");
        assert_eq!(input.bytes(), b"not-yet-valid");
        assert_eq!(input.byte_count(), 13);
    }

    #[test]
    fn equal_bytes_in_different_allocations_are_not_one_incarnation() {
        let left = Vec::from(&b"same-scope"[..]);
        let right = Vec::from(&b"same-scope"[..]);
        let inspected = UntrustedPhysicalArtifact::from_bounded_bytes(&left);
        assert!(inspected.same_incarnation(inspected));
        assert!(!inspected.same_incarnation(UntrustedPhysicalArtifact::from_bounded_bytes(&right)));
    }
}
