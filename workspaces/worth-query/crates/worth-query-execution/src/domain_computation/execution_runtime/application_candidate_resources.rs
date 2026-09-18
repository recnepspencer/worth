use std::num::NonZeroU64;

/// Host limits for one admitted application candidate. Concurrent admission is
/// governed separately by the runtime's shared graph-work capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCandidateResourceProfile {
    maximum_items: NonZeroU64,
    maximum_retained_representation_bytes: NonZeroU64,
    maximum_validator_work: NonZeroU64,
    maximum_operation_model_size: NonZeroU64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCandidateResourceProfileDenial {
    ZeroItems,
    ZeroRetainedRepresentationBytes,
    ZeroValidatorWork,
    ZeroOperationModelSize,
}

impl WorthQueryApplicationCandidateResourceProfile {
    pub fn bounded(
        maximum_items: u64,
        maximum_retained_representation_bytes: u64,
        maximum_validator_work: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        use WorthQueryApplicationCandidateResourceProfileDenial as Denial;
        Ok(Self {
            maximum_items: NonZeroU64::new(maximum_items).ok_or(Denial::ZeroItems)?,
            maximum_retained_representation_bytes: NonZeroU64::new(
                maximum_retained_representation_bytes,
            )
            .ok_or(Denial::ZeroRetainedRepresentationBytes)?,
            maximum_validator_work: NonZeroU64::new(maximum_validator_work)
                .ok_or(Denial::ZeroValidatorWork)?,
            maximum_operation_model_size: NonZeroU64::new(4_096)
                .expect("the default operation model-size ceiling is nonzero"),
        })
    }

    pub fn with_maximum_operation_model_size(
        mut self,
        maximum_operation_model_size: u64,
    ) -> Result<Self, WorthQueryApplicationCandidateResourceProfileDenial> {
        self.maximum_operation_model_size = NonZeroU64::new(maximum_operation_model_size)
            .ok_or(WorthQueryApplicationCandidateResourceProfileDenial::ZeroOperationModelSize)?;
        Ok(self)
    }

    pub const fn maximum_items(self) -> u64 {
        self.maximum_items.get()
    }

    pub const fn maximum_retained_representation_bytes(self) -> u64 {
        self.maximum_retained_representation_bytes.get()
    }

    pub const fn maximum_validator_work(self) -> u64 {
        self.maximum_validator_work.get()
    }

    pub const fn maximum_operation_model_size(self) -> u64 {
        self.maximum_operation_model_size.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_model_size_requires_explicit_nonzero_host_capacity() {
        let profile = WorthQueryApplicationCandidateResourceProfile::bounded(8, 16, 32)
            .unwrap()
            .with_maximum_operation_model_size(32_768)
            .unwrap();
        assert_eq!(profile.maximum_operation_model_size(), 32_768);
        assert_eq!(
            WorthQueryApplicationCandidateResourceProfile::bounded(8, 16, 32)
                .unwrap()
                .with_maximum_operation_model_size(0),
            Err(WorthQueryApplicationCandidateResourceProfileDenial::ZeroOperationModelSize)
        );
    }
}

impl Default for WorthQueryApplicationCandidateResourceProfile {
    fn default() -> Self {
        Self::bounded(4096, 4096, 4096).expect("default candidate limits are nonzero")
    }
}
