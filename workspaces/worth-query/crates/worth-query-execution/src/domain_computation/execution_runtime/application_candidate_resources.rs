use std::num::NonZeroU64;

/// Host limits for one admitted application candidate. Concurrent admission is
/// governed separately by the runtime's shared graph-work capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCandidateResourceProfile {
    maximum_items: Option<NonZeroU64>,
    maximum_retained_representation_bytes: Option<NonZeroU64>,
    maximum_operation_width: Option<NonZeroU64>,
    maximum_producer_dependency_bytes: Option<NonZeroU64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCandidateResourceProfileDenial {
    ZeroItems,
    ZeroRetainedRepresentationBytes,
    ZeroOperationWidth,
    ZeroProducerDependencyBytes,
}

impl WorthQueryApplicationCandidateResourceProfile {
    pub fn policy(
        maximum_items: Option<u64>,
        maximum_retained_representation_bytes: Option<u64>,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        use WorthQueryApplicationCandidateResourceProfileDenial as Denial;
        Ok(Self {
            maximum_items: maximum_items
                .map(|value| NonZeroU64::new(value).ok_or(Denial::ZeroItems))
                .transpose()?,
            maximum_retained_representation_bytes: maximum_retained_representation_bytes
                .map(|value| NonZeroU64::new(value).ok_or(Denial::ZeroRetainedRepresentationBytes))
                .transpose()?,
            maximum_operation_width: None,
            maximum_producer_dependency_bytes: None,
        })
    }

    pub fn with_maximum_producer_dependency_bytes(
        mut self,
        maximum_producer_dependency_bytes: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        self.maximum_producer_dependency_bytes =
            Some(NonZeroU64::new(maximum_producer_dependency_bytes).ok_or(
                WorthQueryApplicationCandidateResourceProfileDenial::ZeroProducerDependencyBytes,
            )?);
        Ok(self)
    }

    pub fn with_maximum_operation_width(
        mut self,
        maximum_operation_width: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        self.maximum_operation_width = Some(
            NonZeroU64::new(maximum_operation_width)
                .ok_or(WorthQueryApplicationCandidateResourceProfileDenial::ZeroOperationWidth)?,
        );
        Ok(self)
    }

    pub fn maximum_items(self) -> Option<u64> {
        self.maximum_items.map(NonZeroU64::get)
    }

    pub fn maximum_retained_representation_bytes(self) -> Option<u64> {
        self.maximum_retained_representation_bytes
            .map(NonZeroU64::get)
    }

    pub fn maximum_operation_width(self) -> Option<u64> {
        self.maximum_operation_width.map(NonZeroU64::get)
    }

    pub fn maximum_producer_dependency_bytes(self) -> Option<u64> {
        self.maximum_producer_dependency_bytes.map(NonZeroU64::get)
    }
}

impl Default for WorthQueryApplicationCandidateResourceProfile {
    fn default() -> Self {
        Self::policy(None, None).expect("absent candidate policy is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_default_and_independent_absence_keep_no_candidate_ceiling() {
        let default = WorthQueryApplicationCandidateResourceProfile::default();
        assert_eq!(default.maximum_items(), None);
        assert_eq!(default.maximum_retained_representation_bytes(), None);
        let items = WorthQueryApplicationCandidateResourceProfile::policy(Some(7), None).unwrap();
        assert_eq!(items.maximum_items(), Some(7));
        assert_eq!(items.maximum_retained_representation_bytes(), None);
        let bytes = WorthQueryApplicationCandidateResourceProfile::policy(None, Some(19)).unwrap();
        assert_eq!(bytes.maximum_items(), None);
        assert_eq!(bytes.maximum_retained_representation_bytes(), Some(19));
        assert_eq!(
            WorthQueryApplicationCandidateResourceProfile::policy(Some(0), None),
            Err(WorthQueryApplicationCandidateResourceProfileDenial::ZeroItems)
        );
        assert_eq!(WorthQueryApplicationCandidateResourceProfile::policy(None, Some(0)), Err(WorthQueryApplicationCandidateResourceProfileDenial::ZeroRetainedRepresentationBytes));
    }

    #[test]
    fn operation_width_requires_explicit_nonzero_host_capacity() {
        let profile = WorthQueryApplicationCandidateResourceProfile::policy(Some(8), Some(16))
            .unwrap()
            .with_maximum_operation_width(32_768)
            .unwrap();
        assert_eq!(profile.maximum_operation_width(), Some(32_768));
        assert_eq!(
            WorthQueryApplicationCandidateResourceProfile::policy(Some(8), Some(16))
                .unwrap()
                .with_maximum_operation_width(0),
            Err(WorthQueryApplicationCandidateResourceProfileDenial::ZeroOperationWidth)
        );
    }

    #[test]
    fn producer_dependency_bytes_require_explicit_nonzero_host_capacity() {
        let profile = WorthQueryApplicationCandidateResourceProfile::policy(Some(8), Some(16))
            .unwrap()
            .with_maximum_producer_dependency_bytes(16 * 1_024 * 1_024)
            .unwrap();
        assert_eq!(
            profile.maximum_producer_dependency_bytes(),
            Some(16 * 1_024 * 1_024)
        );
        assert_eq!(
            WorthQueryApplicationCandidateResourceProfile::policy(Some(8), Some(16))
                .unwrap()
                .with_maximum_producer_dependency_bytes(0),
            Err(WorthQueryApplicationCandidateResourceProfileDenial::ZeroProducerDependencyBytes)
        );
    }
}
