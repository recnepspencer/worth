use std::num::NonZeroU64;

/// Host limits for one admitted application candidate. Concurrent admission is
/// governed separately by the runtime's shared graph-work capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCandidateResourceProfile {
    maximum_items: NonZeroU64,
    maximum_retained_representation_bytes: NonZeroU64,
    maximum_validator_work: Option<NonZeroU64>,
    maximum_operation_width: NonZeroU64,
    maximum_producer_dependency_bytes: NonZeroU64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCandidateResourceProfileDenial {
    ZeroItems,
    ZeroRetainedRepresentationBytes,
    ZeroValidatorWork,
    ZeroOperationWidth,
    ZeroProducerDependencyBytes,
}

impl WorthQueryApplicationCandidateResourceProfile {
    pub fn bounded(
        maximum_items: u64,
        maximum_retained_representation_bytes: u64,
        maximum_validator_work: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        let mut profile =
            Self::physical_resources(maximum_items, maximum_retained_representation_bytes)?;
        profile.maximum_validator_work = Some(
            NonZeroU64::new(maximum_validator_work)
                .ok_or(WorthQueryApplicationCandidateResourceProfileDenial::ZeroValidatorWork)?,
        );
        Ok(profile)
    }

    fn physical_resources(
        maximum_items: u64,
        maximum_retained_representation_bytes: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        use WorthQueryApplicationCandidateResourceProfileDenial as Denial;
        Ok(Self {
            maximum_items: NonZeroU64::new(maximum_items).ok_or(Denial::ZeroItems)?,
            maximum_retained_representation_bytes: NonZeroU64::new(
                maximum_retained_representation_bytes,
            )
            .ok_or(Denial::ZeroRetainedRepresentationBytes)?,
            maximum_validator_work: None,
            maximum_operation_width: NonZeroU64::new(4_096)
                .expect("the default operation width ceiling is nonzero"),
            maximum_producer_dependency_bytes: NonZeroU64::new(4 * 1_024 * 1_024)
                .expect("the default producer dependency ceiling is nonzero"),
        })
    }

    pub fn with_maximum_producer_dependency_bytes(
        mut self,
        maximum_producer_dependency_bytes: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        self.maximum_producer_dependency_bytes = NonZeroU64::new(maximum_producer_dependency_bytes)
            .ok_or(
                WorthQueryApplicationCandidateResourceProfileDenial::ZeroProducerDependencyBytes,
            )?;
        Ok(self)
    }

    pub fn with_maximum_operation_width(
        mut self,
        maximum_operation_width: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        self.maximum_operation_width = NonZeroU64::new(maximum_operation_width)
            .ok_or(WorthQueryApplicationCandidateResourceProfileDenial::ZeroOperationWidth)?;
        Ok(self)
    }

    pub const fn maximum_items(self) -> u64 {
        self.maximum_items.get()
    }

    pub const fn maximum_retained_representation_bytes(self) -> u64 {
        self.maximum_retained_representation_bytes.get()
    }

    /// Removes the host aggregate execution-work budget; physical capacities remain.
    pub const fn without_validator_work_budget(mut self) -> Self {
        self.maximum_validator_work = None;
        self
    }

    pub const fn maximum_validator_work(self) -> Option<u64> {
        match self.maximum_validator_work {
            Some(value) => Some(value.get()),
            None => None,
        }
    }

    pub const fn maximum_operation_width(self) -> u64 {
        self.maximum_operation_width.get()
    }

    pub const fn maximum_producer_dependency_bytes(self) -> u64 {
        self.maximum_producer_dependency_bytes.get()
    }
}

impl Default for WorthQueryApplicationCandidateResourceProfile {
    fn default() -> Self {
        Self::physical_resources(4096, 4096).expect("default candidate capacities are nonzero")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_width_requires_explicit_nonzero_host_capacity() {
        let profile = WorthQueryApplicationCandidateResourceProfile::bounded(8, 16, 32)
            .unwrap()
            .with_maximum_operation_width(32_768)
            .unwrap();
        assert_eq!(profile.maximum_operation_width(), 32_768);
        assert_eq!(
            WorthQueryApplicationCandidateResourceProfile::bounded(8, 16, 32)
                .unwrap()
                .with_maximum_operation_width(0),
            Err(WorthQueryApplicationCandidateResourceProfileDenial::ZeroOperationWidth)
        );
    }

    #[test]
    fn producer_dependency_bytes_require_explicit_nonzero_host_capacity() {
        let profile = WorthQueryApplicationCandidateResourceProfile::bounded(8, 16, 32)
            .unwrap()
            .with_maximum_producer_dependency_bytes(16 * 1_024 * 1_024)
            .unwrap();
        assert_eq!(
            profile.maximum_producer_dependency_bytes(),
            16 * 1_024 * 1_024
        );
        assert_eq!(
            WorthQueryApplicationCandidateResourceProfile::bounded(8, 16, 32)
                .unwrap()
                .with_maximum_producer_dependency_bytes(0),
            Err(WorthQueryApplicationCandidateResourceProfileDenial::ZeroProducerDependencyBytes)
        );
    }
}
