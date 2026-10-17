//! Finite representation limits. These are not request or allocation authority.
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ValueDecodeBounds {
    maximum_wire_bytes: usize,
    maximum_string_bytes: usize,
    maximum_sequence_length: usize,
    maximum_depth: usize,
}

impl ValueDecodeBounds {
    /// Every bound must be finite and nonzero. Depth counts structural nesting.
    pub const fn new(wire: usize, string: usize, sequence: usize, depth: usize) -> Self {
        assert!(wire > 0 && wire < usize::MAX);
        assert!(string > 0 && string < usize::MAX);
        assert!(sequence > 0 && sequence < usize::MAX);
        assert!(depth > 0 && depth < 128);
        Self {
            maximum_wire_bytes: wire,
            maximum_string_bytes: string,
            maximum_sequence_length: sequence,
            maximum_depth: depth,
        }
    }

    pub const fn maximum_wire_bytes(self) -> usize {
        self.maximum_wire_bytes
    }
    pub const fn maximum_string_bytes(self) -> usize {
        self.maximum_string_bytes
    }
    pub const fn maximum_sequence_length(self) -> usize {
        self.maximum_sequence_length
    }
    pub const fn maximum_depth(self) -> usize {
        self.maximum_depth
    }

    pub fn check_string_bytes(self, length: usize) -> Result<(), ValueCodecDenial> {
        if length > self.maximum_string_bytes {
            Err(ValueCodecDenial::StringBytes)
        } else {
            Ok(())
        }
    }

    pub fn descend(mut self) -> Result<Self, ValueCodecDenial> {
        self.maximum_depth = self
            .maximum_depth
            .checked_sub(1)
            .ok_or(ValueCodecDenial::Depth)?;
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueCodecDenial {
    WireBytes,
    StringBytes,
    SequenceLength,
    Depth,
    Allocation,
    Malformed,
    NonCanonical,
}

impl std::fmt::Display for ValueCodecDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WireBytes => "canonical value exceeds its wire byte bound",
            Self::StringBytes => "canonical value exceeds its string byte bound",
            Self::SequenceLength => "canonical value exceeds its sequence length bound",
            Self::Depth => "canonical value exceeds its nesting depth bound",
            Self::Allocation => "canonical value allocation refused",
            Self::Malformed => "canonical value does not match its declared wire grammar",
            Self::NonCanonical => "canonical value bytes are not in their declared canonical form",
        })
    }
}
impl std::error::Error for ValueCodecDenial {}
