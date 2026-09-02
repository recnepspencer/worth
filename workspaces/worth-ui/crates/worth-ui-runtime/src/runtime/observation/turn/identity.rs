#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiObservationTurnIdentity(u64);

impl UiObservationTurnIdentity {
    pub(super) const fn issued_by_runtime(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(crate) const fn for_test(value: u64) -> Self {
        Self(value)
    }
}
