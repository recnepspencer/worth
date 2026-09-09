//! Atlas snapshot and destruction boundary.

use super::ownership::{AtlasCore, UiNativeTextAtlas};
use super::recovery::{UiNativeTextAtlasGeneration, UiNativeTextAtlasRecovery};
use super::settlement::UiNativeTextAtlasSnapshot;

impl UiNativeTextAtlas {
    #[cfg(feature = "certification-support")]
    pub(crate) fn can_mutate_for_reconstruction(&self) -> bool {
        self.core
            .try_borrow_mut()
            .is_ok_and(|core| next_clear_generation(&core).is_some())
    }

    pub(crate) fn snapshot(&self) -> UiNativeTextAtlasSnapshot {
        let core = self.core.borrow();
        UiNativeTextAtlasSnapshot {
            generation: core.generation,
            alpha_pages: core.alpha.pages.len() as u32,
            color_pages: core.color.pages.len() as u32,
            alpha_entries: core.alpha.entries.len() as u32,
            color_entries: core.color.entries.len() as u32,
            pins: core.pins.len() as u32,
            staging_bytes: 0,
            reservation_active: core.reservation.is_some(),
        }
    }

    #[cfg(test)]
    pub(crate) fn census(&self) -> super::census::UiNativeTextAtlasCensus {
        super::census::UiNativeTextAtlasCensus::from_snapshot_with_posture(
            self.snapshot(),
            0,
            super::census::UiNativeTextAtlasPhysicalPosture::default(),
        )
    }

    pub(crate) fn clear(&self) -> bool {
        let Ok(mut core) = self.core.try_borrow_mut() else {
            return false;
        };
        let Some(generation) = next_clear_generation(&core) else {
            return false;
        };
        core.reset_for_reconstruction(generation);
        true
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn remove_pins(
        &self,
        requests: &[worth_ui_host_contract::UiGlyphRasterPinRequest],
    ) -> bool {
        let Ok(mut core) = self.core.try_borrow_mut() else {
            return false;
        };
        for request in requests {
            core.pins.remove(&super::ownership::PinIdentity::new(
                request.layout_identity(),
                request.key(),
            ));
        }
        true
    }

    pub(crate) fn recover(&self, recovery: &UiNativeTextAtlasRecovery) -> bool {
        let Ok(mut core) = self.core.try_borrow_mut() else {
            return false;
        };
        if !recovery_matches(&core, recovery) {
            return false;
        }
        core.reset_for_reconstruction(recovery.generation());
        true
    }

    pub(crate) fn recovery_matches(&self, recovery: &UiNativeTextAtlasRecovery) -> bool {
        self.core
            .try_borrow()
            .is_ok_and(|core| recovery_matches(&core, recovery))
    }
}

fn recovery_matches(core: &AtlasCore, recovery: &UiNativeTextAtlasRecovery) -> bool {
    core.quarantined
        && recovery.generation().get() == core.generation.get().saturating_add(1)
        && recovery.lineage_identity() == core.lineage
}

fn next_clear_generation(core: &AtlasCore) -> Option<UiNativeTextAtlasGeneration> {
    core.generation
        .get()
        .checked_add(1)
        .and_then(UiNativeTextAtlasGeneration::new)
}
