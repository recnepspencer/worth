use std::num::NonZeroUsize;

/// Explicit limits on new history subject capture, including temporary/native selection overlap.
/// Existing disclosed tree, canonical parameters and retained root data remain shared custody.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryApplicationHistoryCaptureLimits {
    pub maximum_subjects: NonZeroUsize,
    pub maximum_aspects: NonZeroUsize,
    /// Each disclosed node and each native metadata/subject resolution is one visit.
    pub maximum_source_visits: NonZeroUsize,
    pub maximum_owned_bytes: NonZeroUsize,
}
