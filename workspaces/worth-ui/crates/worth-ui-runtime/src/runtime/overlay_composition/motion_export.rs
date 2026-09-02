use super::extent::{UiOverlayMotionBinding, UiOverlayMotionSnapshot};
use super::snapshot::UiOverlayApplicationGeneration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayMotionOwnerExport {
    generation: UiOverlayApplicationGeneration,
    snapshot: UiOverlayMotionSnapshot,
}

impl UiOverlayMotionOwnerExport {
    pub(super) fn from_prepared(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        owner_revision: u64,
        rows: impl IntoIterator<Item = UiOverlayMotionBinding>,
    ) -> Result<Self, ()> {
        let generation = UiOverlayApplicationGeneration::from_prepared(generation);
        Ok(Self {
            generation,
            snapshot: UiOverlayMotionSnapshot::seal(owner_revision, rows)?,
        })
    }

    pub(super) fn generation(&self) -> &UiOverlayApplicationGeneration {
        &self.generation
    }

    pub(super) fn snapshot(&self) -> &UiOverlayMotionSnapshot {
        &self.snapshot
    }
}
