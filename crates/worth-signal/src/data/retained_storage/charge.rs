use super::RetainedStoragePreparationDenial;

/// Checked bytes of retained representation, excluding allocator bookkeeping
/// and process RSS. Shared payloads may be conservatively charged per reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct RetainedStorageCharge(u64);

impl RetainedStorageCharge {
    pub(crate) const ZERO: Self = Self(0);

    pub(crate) const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    pub(crate) const fn bytes(self) -> u64 {
        self.0
    }

    pub(crate) fn capacity<T>(count: usize) -> Result<Self, RetainedStoragePreparationDenial> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)?;
        Ok(Self(bytes))
    }

    pub(crate) fn checked_add(self, other: Self) -> Result<Self, RetainedStoragePreparationDenial> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)
    }

    pub(crate) fn checked_mul(
        self,
        count: usize,
    ) -> Result<Self, RetainedStoragePreparationDenial> {
        let count =
            u64::try_from(count).map_err(|_| RetainedStoragePreparationDenial::ChargeOverflow)?;
        self.0
            .checked_mul(count)
            .map(Self)
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)
    }

    pub(crate) fn checked_sub(self, other: Self) -> Result<Self, RetainedStoragePreparationDenial> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(RetainedStoragePreparationDenial::ChargeUnderflow)
    }
}
