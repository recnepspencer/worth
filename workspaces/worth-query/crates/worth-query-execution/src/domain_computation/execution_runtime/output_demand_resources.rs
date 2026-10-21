use std::num::NonZeroUsize;
/// An explicit finite convenience-settlement step policy. Absence keeps advancing
/// genuine Progressed outcomes, and returns on native waiting or interruption.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct WorthQueryOutputDemandResourceProfile {
    settlement_attempts: Option<NonZeroUsize>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandLimits {
    settlement_attempts: Option<usize>,
}
impl WorthQueryOutputDemandResourceProfile {
    pub const fn standard() -> Self {
        Self {
            settlement_attempts: None,
        }
    }
    pub const fn new(settlement_attempts: NonZeroUsize) -> Self {
        Self {
            settlement_attempts: Some(settlement_attempts),
        }
    }
    pub fn limits(self) -> WorthQueryOutputDemandLimits {
        WorthQueryOutputDemandLimits {
            settlement_attempts: self.settlement_attempts.map(NonZeroUsize::get),
        }
    }
    pub fn constrain(
        self,
        requested: WorthQueryOutputDemandLimits,
    ) -> WorthQueryOutputDemandLimits {
        self.limits().restricted(requested.settlement_attempts)
    }
}
impl WorthQueryOutputDemandLimits {
    pub fn restricted(self, settlement_attempts: Option<usize>) -> Self {
        Self {
            settlement_attempts: self
                .settlement_attempts
                .into_iter()
                .chain(settlement_attempts)
                .min(),
        }
    }
    pub const fn settlement_attempts(self) -> Option<usize> {
        self.settlement_attempts
    }
}
