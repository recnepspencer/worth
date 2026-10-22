//! Optional representation policies. These are not allocation authority.
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ValueDecodeBounds {
    maximum_wire_bytes: Option<usize>,
    maximum_string_bytes: Option<usize>,
    maximum_sequence_length: Option<usize>,
    maximum_depth: Option<usize>,
}

impl ValueDecodeBounds {
    /// Every bound must be finite and nonzero. Depth counts structural nesting.
    pub const fn new(wire: usize, string: usize, sequence: usize, depth: usize) -> Self {
        assert!(wire > 0 && wire < usize::MAX);
        assert!(string > 0 && string < usize::MAX);
        assert!(sequence > 0 && sequence < usize::MAX);
        assert!(depth > 0 && depth < usize::MAX);
        Self {
            maximum_wire_bytes: Some(wire),
            maximum_string_bytes: Some(string),
            maximum_sequence_length: Some(sequence),
            maximum_depth: Some(depth),
        }
    }

    /// No guessed representation ceiling; grammar and domain admission still apply.
    pub const fn unrestricted() -> Self {
        Self {
            maximum_wire_bytes: None,
            maximum_string_bytes: None,
            maximum_sequence_length: None,
            maximum_depth: None,
        }
    }

    pub const fn maximum_wire_bytes(self) -> Option<usize> {
        self.maximum_wire_bytes
    }
    /// Intersect the caller policy with the domain's actual component policy.
    pub fn with_string_limit(mut self, maximum: usize) -> Self {
        assert!(maximum > 0 && maximum < usize::MAX);
        self.maximum_string_bytes = Some(
            self.maximum_string_bytes
                .map_or(maximum, |old| old.min(maximum)),
        );
        self
    }
    pub fn with_sequence_limit(mut self, maximum: usize) -> Self {
        assert!(maximum > 0 && maximum < usize::MAX);
        self.maximum_sequence_length = Some(
            self.maximum_sequence_length
                .map_or(maximum, |old| old.min(maximum)),
        );
        self
    }
    pub const fn maximum_string_bytes(self) -> Option<usize> {
        self.maximum_string_bytes
    }
    pub const fn maximum_sequence_length(self) -> Option<usize> {
        self.maximum_sequence_length
    }
    pub const fn maximum_depth(self) -> Option<usize> {
        self.maximum_depth
    }

    pub fn check_string_bytes(self, length: usize) -> Result<(), ValueCodecDenial> {
        if self
            .maximum_string_bytes
            .is_some_and(|maximum| length > maximum)
        {
            Err(ValueCodecDenial::StringBytes)
        } else {
            Ok(())
        }
    }

    pub fn descend(mut self) -> Result<Self, ValueCodecDenial> {
        if let Some(depth) = self.maximum_depth {
            self.maximum_depth = Some(depth.checked_sub(1).ok_or(ValueCodecDenial::Depth)?);
        }
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
    Admission,
    UnsupportedAdmission,
    UnsupportedShape,
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
            Self::Admission => "canonical value request admission refused",
            Self::UnsupportedAdmission => {
                "canonical value owner does not provide admitted decoding"
            }
            Self::UnsupportedShape => {
                "canonical grammar has an unguarded recursive cycle or ambiguous optional payload"
            }
        })
    }
}
impl std::error::Error for ValueCodecDenial {}
