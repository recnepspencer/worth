/// Finite resources reserved before a fixed-shape candidate is constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCandidateResourceCeiling {
    maximum_retained_representation_bytes: usize,
    maximum_validator_work: Option<usize>,
}

impl ApplicationCandidateResourceCeiling {
    /// Bounds retained representation; Query derives validator work from the
    /// installed invariant closure. This does not relax host capacity admission.
    pub const fn representation_bytes(maximum_retained_representation_bytes: usize) -> Self {
        Self {
            maximum_retained_representation_bytes,
            maximum_validator_work: None,
        }
    }

    pub const fn bounded(
        maximum_retained_representation_bytes: usize,
        maximum_validator_work: usize,
    ) -> Self {
        Self {
            maximum_retained_representation_bytes,
            maximum_validator_work: Some(maximum_validator_work),
        }
    }

    pub const fn maximum_retained_representation_bytes(self) -> usize {
        self.maximum_retained_representation_bytes
    }

    /// A deliberate validator cap, or no additional application restriction.
    pub const fn maximum_validator_work(self) -> Option<usize> {
        self.maximum_validator_work
    }
}
