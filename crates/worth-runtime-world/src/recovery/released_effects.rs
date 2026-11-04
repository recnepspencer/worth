//! Actual release returns terminal original data and its still-owned cleanup.
use super::{ProductUnpublishedCleanup, WorldReleasedOriginal};

#[derive(Debug)]
pub struct WorldReleasedEffects {
    original: WorldReleasedOriginal,
    cleanup: ProductUnpublishedCleanup,
}
impl WorldReleasedEffects {
    pub(crate) fn new(original: WorldReleasedOriginal, cleanup: ProductUnpublishedCleanup) -> Self {
        Self { original, cleanup }
    }
    pub fn original(&self) -> &WorldReleasedOriginal {
        &self.original
    }
    pub fn into_parts(self) -> (WorldReleasedOriginal, ProductUnpublishedCleanup) {
        (self.original, self.cleanup)
    }
}
