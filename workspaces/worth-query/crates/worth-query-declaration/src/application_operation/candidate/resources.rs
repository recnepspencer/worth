/// Finite resources reserved before a fixed-shape candidate is constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCandidateResourceCeiling {
    maximum_retained_representation_bytes: usize,
}

impl ApplicationCandidateResourceCeiling {
    /// Bounds the retained candidate representation.
    pub const fn representation_bytes(maximum_retained_representation_bytes: usize) -> Self {
        Self {
            maximum_retained_representation_bytes,
        }
    }

    pub const fn maximum_retained_representation_bytes(self) -> usize {
        self.maximum_retained_representation_bytes
    }
}
