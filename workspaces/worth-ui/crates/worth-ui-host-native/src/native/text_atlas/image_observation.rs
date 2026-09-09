//! Atlas-owned identity of readable image metadata, without presentation authority.

use super::ownership::UiNativeTextAtlas;
use super::recovery::{UiNativeTextAtlasGeneration, UiNativeTextAtlasLineageIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeTextAtlasImageObservation {
    lineage: UiNativeTextAtlasLineageIdentity,
    generation: UiNativeTextAtlasGeneration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeTextAtlasImageObservationDenial {
    Reserved,
    Quarantined,
    Stale,
}

impl UiNativeTextAtlas {
    pub(crate) fn observe_images(
        &self,
    ) -> Result<UiNativeTextAtlasImageObservation, UiNativeTextAtlasImageObservationDenial> {
        let core = self.core.borrow();
        if core.quarantined {
            return Err(UiNativeTextAtlasImageObservationDenial::Quarantined);
        }
        if core.reservation.is_some() {
            return Err(UiNativeTextAtlasImageObservationDenial::Reserved);
        }
        Ok(UiNativeTextAtlasImageObservation {
            lineage: core.lineage,
            generation: core.generation,
        })
    }

    pub(crate) fn validate_images(
        &self,
        observation: UiNativeTextAtlasImageObservation,
    ) -> Result<(), UiNativeTextAtlasImageObservationDenial> {
        if self.observe_images()? != observation {
            return Err(UiNativeTextAtlasImageObservationDenial::Stale);
        }
        Ok(())
    }
}
