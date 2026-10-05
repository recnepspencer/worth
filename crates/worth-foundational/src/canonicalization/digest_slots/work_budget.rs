#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalDigestWorkBudget {
    maximum_entry_count: u32,
    maximum_encoded_bytes: usize,
}

impl CanonicalDigestWorkBudget {
    const STANDARD_MAXIMUM_ENTRY_COUNT: u32 = 4_096;
    const STANDARD_MAXIMUM_ENCODED_BYTES: usize = 4 * 1_024 * 1_024;

    /// Bound entry storage by an encoded-byte ceiling without imposing an
    /// independent cardinality limit on otherwise admissible material.
    ///
    /// Each encoded entry includes the `entry.domain` and `entry.kind` tokens.
    /// Their labels and framing alone occupy at least 30 bytes, even with empty
    /// values. Exact encoded-byte admission remains the digest owner's job.
    pub const fn for_encoded_byte_ceiling(maximum_encoded_bytes: usize) -> Option<Self> {
        let entries = maximum_encoded_bytes / Self::MINIMUM_ENCODED_ENTRY_BYTES;
        let maximum_entry_count = if entries == 0 {
            1
        } else if entries > u32::MAX as usize {
            u32::MAX
        } else {
            entries as u32
        };
        Self::new(maximum_entry_count, maximum_encoded_bytes)
    }

    pub(crate) const MINIMUM_ENCODED_ENTRY_BYTES: usize =
        "entry.domain".len() + 4 + "entry.kind".len() + 4;

    pub const fn new(maximum_entry_count: u32, maximum_encoded_bytes: usize) -> Option<Self> {
        if maximum_entry_count == 0 || maximum_encoded_bytes == 0 {
            None
        } else {
            Some(Self {
                maximum_entry_count,
                maximum_encoded_bytes,
            })
        }
    }

    pub const fn maximum_entry_count(self) -> u32 {
        self.maximum_entry_count
    }

    pub const fn maximum_encoded_bytes(self) -> usize {
        self.maximum_encoded_bytes
    }

    pub const fn standard() -> Self {
        Self {
            maximum_entry_count: Self::STANDARD_MAXIMUM_ENTRY_COUNT,
            maximum_encoded_bytes: Self::STANDARD_MAXIMUM_ENCODED_BYTES,
        }
    }
}
