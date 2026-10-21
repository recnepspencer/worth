use std::num::NonZeroUsize;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationQueryLimits {
    maximum_results: NonZeroUsize,
}
impl WorthQueryInstalledApplicationQueryLimits {
    pub(crate) const fn new(maximum_results: NonZeroUsize) -> Self {
        Self { maximum_results }
    }
    pub const fn maximum_results(self) -> NonZeroUsize {
        self.maximum_results
    }
    pub fn resolve(self) -> WorthQueryResolvedApplicationQueryLimits {
        WorthQueryResolvedApplicationQueryLimits {
            maximum_results: self.maximum_results,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryResolvedApplicationQueryLimits {
    maximum_results: NonZeroUsize,
}
impl WorthQueryResolvedApplicationQueryLimits {
    pub const fn maximum_results(self) -> NonZeroUsize {
        self.maximum_results
    }
    pub fn narrow(
        self,
        maximum_results: NonZeroUsize,
    ) -> Result<Self, WorthQueryApplicationQueryLimitDenial> {
        if maximum_results > self.maximum_results {
            return Err(WorthQueryApplicationQueryLimitDenial::MaximumResultsWidened);
        }
        Ok(Self { maximum_results })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryLimitDenial {
    MaximumResultsWidened,
}
impl std::fmt::Display for WorthQueryApplicationQueryLimitDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "application query cardinality denied: {self:?}")
    }
}
impl std::error::Error for WorthQueryApplicationQueryLimitDenial {}
