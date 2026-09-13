use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationQueryLimits {
    maximum_results: NonZeroUsize,
    maximum_work: NonZeroUsize,
}

impl WorthQueryInstalledApplicationQueryLimits {
    pub(crate) const fn new(maximum_results: NonZeroUsize, maximum_work: NonZeroUsize) -> Self {
        Self {
            maximum_results,
            maximum_work,
        }
    }

    pub const fn maximum_results(self) -> NonZeroUsize {
        self.maximum_results
    }

    pub const fn maximum_work(self) -> NonZeroUsize {
        self.maximum_work
    }

    pub fn narrow(
        self,
        maximum_results: NonZeroUsize,
        maximum_work: NonZeroUsize,
    ) -> Result<Self, WorthQueryApplicationQueryLimitDenial> {
        if maximum_results.get() > self.maximum_results.get() {
            return Err(WorthQueryApplicationQueryLimitDenial::MaximumResultsWidened);
        }
        if maximum_work.get() > self.maximum_work.get() {
            return Err(WorthQueryApplicationQueryLimitDenial::MaximumWorkWidened);
        }
        Ok(Self::new(maximum_results, maximum_work))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryLimitDenial {
    MaximumResultsWidened,
    MaximumWorkWidened,
}

impl std::fmt::Display for WorthQueryApplicationQueryLimitDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application query limit denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryApplicationQueryLimitDenial {}
