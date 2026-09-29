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
        host_maximum_work: NonZeroUsize,
    ) -> WorthQueryResolvedApplicationQueryLimits {
        WorthQueryResolvedApplicationQueryLimits {
            maximum_results: self.maximum_results,
            maximum_work: self
                .maximum_work
                .map_or(host_maximum_work, |cap| cap.min(host_maximum_work)),
        }
    }
}

/// Concrete finite controls after applying runtime policy to installed meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryResolvedApplicationQueryLimits {
    maximum_results: NonZeroUsize,
    maximum_work: NonZeroUsize,
}

impl WorthQueryResolvedApplicationQueryLimits {
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
        assert_eq!(ordinary.resolve(nz(100)).maximum_work(), nz(100));
        let explicit = WorthQueryInstalledApplicationQueryLimits::new(nz(4), Some(nz(30)));
        assert_eq!(explicit.resolve(nz(100)).maximum_work(), nz(30));
        assert_eq!(explicit.resolve(nz(20)).maximum_work(), nz(20));
        assert_eq!(
            ordinary.resolve(nz(100)).narrow(nz(4), nz(101)),
            Err(WorthQueryApplicationQueryLimitDenial::MaximumWorkWidened)
        );
        assert_eq!(
            ordinary.resolve(nz(100)).narrow(nz(5), nz(100)),
            Err(WorthQueryApplicationQueryLimitDenial::MaximumResultsWidened)
        );
        assert_eq!(
            explicit.resolve(nz(100)).narrow(nz(4), nz(31)),
            Err(WorthQueryApplicationQueryLimitDenial::MaximumWorkWidened)
        );
        assert!(ordinary.resolve(nz(100)).narrow(nz(2), nz(10)).is_ok());
    }
}
