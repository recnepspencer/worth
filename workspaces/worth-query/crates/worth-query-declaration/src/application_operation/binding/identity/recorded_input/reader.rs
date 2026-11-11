//! Concrete cursor, scalar grammar and caller-owned representation bounds.
use super::super::encoder::tag;
use worth_foundational::facade::{ValueCodecDenial, ValueDecodeBounds};
#[derive(Clone, Copy, Debug)]
pub(super) struct CanonicalInputDecodeError(pub ValueCodecDenial);
impl std::fmt::Display for CanonicalInputDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for CanonicalInputDecodeError {}
impl serde::de::Error for CanonicalInputDecodeError {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self(ValueCodecDenial::Malformed)
    }
}
pub(super) type Result<T> = std::result::Result<T, CanonicalInputDecodeError>;
pub(super) struct Reader<'a> {
    pub(super) remaining: &'a [u8],
    pub(super) bounds: ValueDecodeBounds,
}
impl<'a> Reader<'a> {
    pub(super) fn new(bytes: &'a [u8], bounds: ValueDecodeBounds) -> Self {
        Self {
            remaining: bytes,
            bounds,
        }
    }
    pub(super) fn finished(&self) -> Result<()> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(CanonicalInputDecodeError(ValueCodecDenial::NonCanonical))
        }
    }
    pub(super) fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let value = self
            .remaining
            .get(..count)
            .ok_or(CanonicalInputDecodeError(ValueCodecDenial::Malformed))?;
        self.remaining = &self.remaining[count..];
        Ok(value)
    }
    pub(super) fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub(super) fn require(&mut self, tag: u8) -> Result<()> {
        if self.byte()? == tag {
            Ok(())
        } else {
            Err(CanonicalInputDecodeError(ValueCodecDenial::Malformed))
        }
    }
    pub(super) fn integer(&mut self) -> Result<u128> {
        let mut value = 0_u128;
        for group in 0..19 {
            let byte = self.byte()?;
            let payload = byte & 127;
            if group == 18 && payload > 3 {
                return Err(CanonicalInputDecodeError(ValueCodecDenial::Malformed));
            }
            value |= u128::from(payload) << (group * 7);
            if byte & 128 == 0 {
                if group > 0 && payload == 0 {
                    return Err(CanonicalInputDecodeError(ValueCodecDenial::NonCanonical));
                }
                return Ok(value);
            }
        }
        Err(CanonicalInputDecodeError(ValueCodecDenial::Malformed))
    }
    pub(super) fn unsigned(&mut self) -> Result<u128> {
        self.require(tag::UNSIGNED)?;
        self.integer()
    }
    pub(super) fn signed(&mut self) -> Result<i128> {
        self.require(tag::SIGNED)?;
        let value = self.integer()?;
        Ok(((value >> 1) as i128) ^ -((value & 1) as i128))
    }
    pub(super) fn text(&mut self) -> Result<&'a str> {
        let bytes = self.bytes()?;
        self.bounds
            .check_string_bytes(bytes.len())
            .map_err(CanonicalInputDecodeError)?;
        std::str::from_utf8(bytes)
            .map_err(|_| CanonicalInputDecodeError(ValueCodecDenial::Malformed))
    }
    pub(super) fn bytes(&mut self) -> Result<&'a [u8]> {
        let length = usize::try_from(self.integer()?)
            .map_err(|_| CanonicalInputDecodeError(ValueCodecDenial::WireBytes))?;
        self.take(length)
    }
    pub(super) fn name(&mut self, expected: &str) -> Result<()> {
        if self.text()? == expected {
            Ok(())
        } else {
            Err(CanonicalInputDecodeError(ValueCodecDenial::NonCanonical))
        }
    }
    pub(super) fn enter(&mut self) -> Result<ValueDecodeBounds> {
        let prior = self.bounds;
        self.bounds = prior.descend().map_err(CanonicalInputDecodeError)?;
        Ok(prior)
    }
    pub(super) fn member(&mut self, count: &mut usize) -> Result<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => {
                *count = count
                    .checked_add(1)
                    .ok_or(CanonicalInputDecodeError(ValueCodecDenial::SequenceLength))?;
                if self
                    .bounds
                    .maximum_sequence_length()
                    .is_some_and(|limit| *count > limit)
                {
                    return Err(CanonicalInputDecodeError(ValueCodecDenial::SequenceLength));
                }
                Ok(true)
            }
            _ => Err(CanonicalInputDecodeError(ValueCodecDenial::Malformed)),
        }
    }
}
