use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationQueryLimits {
    maximum_results: NonZeroUsize,
    maximum_work: Option<NonZeroUsize>,
}

impl WorthQueryInstalledApplicationQueryLimits {
    pub(crate) const fn new(
        maximum_results: NonZeroUsize,
        maximum_work: Option<NonZeroUsize>,
    ) -> Self {
        Self {
            maximum_results,
            maximum_work,
        }
    }

    pub const fn maximum_results(self) -> NonZeroUsize {
        self.maximum_results
    }

    pub const fn maximum_work(self) -> Option<NonZeroUsize> {
        self.maximum_work
    }

    /// Resolves operational policy without widening a declared explicit cap.
    pub fn resolve(
        self,
        host_maximum_work: Option<NonZeroUsize>,
    ) -> WorthQueryResolvedApplicationQueryLimits {
        WorthQueryResolvedApplicationQueryLimits {
            maximum_results: self.maximum_results,
            maximum_work: self.maximum_work.into_iter().chain(host_maximum_work).min(),
        }
    }
}

/// Resolved controls after applying optional runtime work policy to installed meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryResolvedApplicationQueryLimits {
    maximum_results: NonZeroUsize,
    maximum_work: Option<NonZeroUsize>,
}

impl WorthQueryResolvedApplicationQueryLimits {
    pub const fn maximum_results(self) -> NonZeroUsize {
        self.maximum_results
    }
    pub const fn maximum_work(self) -> Option<NonZeroUsize> {
        self.maximum_work
    }

    pub fn narrow(
        self,
        maximum_results: NonZeroUsize,
        maximum_work: Option<NonZeroUsize>,
    ) -> Result<Self, WorthQueryApplicationQueryLimitDenial> {
        if maximum_results.get() > self.maximum_results.get() {
            return Err(WorthQueryApplicationQueryLimitDenial::MaximumResultsWidened);
        }
        if self
            .maximum_work
            .is_some_and(|installed| maximum_work.is_none_or(|requested| requested > installed))
        {
            return Err(WorthQueryApplicationQueryLimitDenial::MaximumWorkWidened);
        }
        Ok(Self {
            maximum_results,
            maximum_work,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    fn nz(value: usize) -> NonZeroUsize {
        NonZeroUsize::new(value).unwrap()
    }

    #[test]
    fn runtime_policy_resolves_absence_and_never_widens_explicit_caps() {
        let ordinary = WorthQueryInstalledApplicationQueryLimits::new(nz(4), None);
        assert_eq!(ordinary.maximum_work(), None);
        assert_eq!(ordinary.resolve(None).maximum_work(), None);
        assert_eq!(
            ordinary.resolve(Some(nz(100))).maximum_work(),
            Some(nz(100))
        );
        let explicit = WorthQueryInstalledApplicationQueryLimits::new(nz(4), Some(nz(30)));
        assert_eq!(explicit.resolve(Some(nz(100))).maximum_work(), Some(nz(30)));
        assert_eq!(explicit.resolve(Some(nz(20))).maximum_work(), Some(nz(20)));
        assert_eq!(
            ordinary.resolve(Some(nz(100))).narrow(nz(4), Some(nz(101))),
            Err(WorthQueryApplicationQueryLimitDenial::MaximumWorkWidened)
        );
        assert_eq!(
            ordinary.resolve(Some(nz(100))).narrow(nz(5), Some(nz(100))),
            Err(WorthQueryApplicationQueryLimitDenial::MaximumResultsWidened)
        );
        assert_eq!(
            explicit.resolve(Some(nz(100))).narrow(nz(4), Some(nz(31))),
            Err(WorthQueryApplicationQueryLimitDenial::MaximumWorkWidened)
        );
        assert!(ordinary
            .resolve(Some(nz(100)))
            .narrow(nz(2), Some(nz(10)))
            .is_ok());
        assert_eq!(
            explicit.resolve(None).narrow(nz(4), None),
            Err(WorthQueryApplicationQueryLimitDenial::MaximumWorkWidened)
        );
    }
}
