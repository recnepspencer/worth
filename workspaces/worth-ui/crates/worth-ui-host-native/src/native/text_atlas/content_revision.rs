//! Identity of committed image metadata for diagnostic cache invalidation.
//! This is not permission to read, upload, pin, or present atlas images.
use super::recovery::{UiNativeTextAtlasGeneration, UiNativeTextAtlasLineageIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeTextAtlasContentRevision {
    lineage: UiNativeTextAtlasLineageIdentity,
    generation: UiNativeTextAtlasGeneration,
}

impl super::UiNativeTextAtlas {
    pub(crate) fn committed_content_revision(&self) -> UiNativeTextAtlasContentRevision {
        let core = self.core.borrow();
        // Reservations hold separate candidate stores. Only settlement/reset
        // changes committed metadata and its existing generation.
        UiNativeTextAtlasContentRevision {
            lineage: core.lineage,
            generation: core.generation,
        }
    }
}
