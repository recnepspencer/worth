use std::num::NonZeroUsize;

/// Aggregate host custody for completed Query evidence, independent of a
/// candidate's temporary representation and of output-demand retention.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCompletedEvidenceResourceProfile {
    retained_bytes: NonZeroUsize,
}

impl WorthQueryCompletedEvidenceResourceProfile {
    pub const fn bounded(retained_bytes: NonZeroUsize) -> Self {
        Self { retained_bytes }
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes.get()
    }

    pub(crate) const fn standard() -> Self {
        Self::bounded(NonZeroUsize::new(4 * 1_024 * 1_024).unwrap())
    }
}

impl Default for WorthQueryCompletedEvidenceResourceProfile {
    fn default() -> Self {
        Self::standard()
    }
}
