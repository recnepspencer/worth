use super::extent::{UiOverlayMotionBinding, UiOverlayMotionSnapshot};
use super::snapshot::UiOverlayApplicationGeneration;
use crate::runtime::motion::UiMotionOverlayOwnerExport;
use crate::runtime::portal::UiPortalOverlayBindingOwnerExport;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayMotionOwnerExport {
    generation: UiOverlayApplicationGeneration,
    snapshot: UiOverlayMotionSnapshot,
}

impl UiOverlayMotionOwnerExport {
    pub(super) fn from_owner(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        source: &UiMotionOverlayOwnerExport,
        bindings: &UiPortalOverlayBindingOwnerExport,
    ) -> Result<Self, ()> {
        let mut rows = Vec::with_capacity(bindings.rows().len());
        for binding in bindings.rows() {
            let target = crate::runtime::motion::UiMotionTargetIdentity::from_family_owner(
                bindings.runtime_surface(),
                binding.portal().owner().mounted_instance_identity(),
                binding.portal().diagnostic_value(),
            );
            if let Ok(index) = source
                .rows()
                .binary_search_by_key(&target, |row| row.target())
            {
                rows.push(UiOverlayMotionBinding::new(
                    binding.declaration(),
                    binding.portal(),
                    source.rows()[index].revision(),
                ));
            }
        }
        Ok(Self {
            generation: UiOverlayApplicationGeneration::from_prepared(generation),
            snapshot: UiOverlayMotionSnapshot::seal(source.owner_revision(), rows)?,
        })
    }

    #[cfg(test)]
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
