/// Caller-owned destination for canonical material. Encoding grants no
/// readiness, comparison, or identity authority and makes no allocations itself.
/// A refusal leaves the caller's previously accepted prefix in its destination.
pub trait CanonicalMaterialSink {
    type Error;

    fn append(&mut self, value: &str) -> Result<(), Self::Error>;
    fn admit_work(&mut self, work: usize) -> Result<(), Self::Error>;
    fn accounting_overflow(&mut self) -> Self::Error;
}

/// Checked encoded length, without constructing canonical text or admitting
/// storage. The eventual allocation owner must separately reserve its backing.
#[derive(Debug, Default)]
pub struct CanonicalMaterialByteCount {
    bytes: usize,
}

impl CanonicalMaterialByteCount {
    pub const fn new() -> Self {
        Self { bytes: 0 }
    }

    pub const fn encoded_bytes(&self) -> usize {
        self.bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalMaterialLengthOverflow;

impl std::fmt::Display for CanonicalMaterialLengthOverflow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("canonical material length exceeds the addressable range")
    }
}

impl std::error::Error for CanonicalMaterialLengthOverflow {}

impl CanonicalMaterialSink for CanonicalMaterialByteCount {
    type Error = CanonicalMaterialLengthOverflow;

    fn append(&mut self, value: &str) -> Result<(), Self::Error> {
        self.bytes = self
            .bytes
            .checked_add(value.len())
            .ok_or(CanonicalMaterialLengthOverflow)?;
        Ok(())
    }

    fn admit_work(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn accounting_overflow(&mut self) -> Self::Error {
        CanonicalMaterialLengthOverflow
    }
}
