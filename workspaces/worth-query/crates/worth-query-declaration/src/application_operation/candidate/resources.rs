/// Finite resources reserved before a fixed-shape candidate is constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCandidateResourceCeiling {
    maximum_retained_representation_bytes: usize,
    maximum_validator_work: usize,
}

impl ApplicationCandidateResourceCeiling {
    pub const fn bounded(
        maximum_retained_representation_bytes: usize,
        maximum_validator_work: usize,
    ) -> Self {
        Self {
            maximum_retained_representation_bytes,
            maximum_validator_work,
        }
    }

    pub const fn maximum_retained_representation_bytes(self) -> usize {
        self.maximum_retained_representation_bytes
    }

    pub const fn maximum_validator_work(self) -> usize {
        self.maximum_validator_work
    }
}
