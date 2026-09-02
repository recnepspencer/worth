use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;
use worth_ui_host_contract::UiMountedPresentationAttemptIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiApplicationPresentationOwnerExport {
    generation: WorthUiPreparedApplicationGenerationIdentity,
    presentation: UiMountedPresentationAttemptIdentity,
    revision: u64,
}

impl UiApplicationPresentationOwnerExport {
    pub(crate) fn generation(&self) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.generation
    }

    pub(crate) const fn presentation(&self) -> UiMountedPresentationAttemptIdentity {
        self.presentation
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }
}

impl super::UiApplicationPresentationState {
    /// Convert the current presentation owner's sealed revision and the
    /// already-issued attempt identity into a value-only Gate 1 export.
    pub(crate) fn overlay_owner_export(
        &self,
        generation: WorthUiPreparedApplicationGenerationIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
    ) -> UiApplicationPresentationOwnerExport {
        UiApplicationPresentationOwnerExport {
            generation,
            presentation,
            revision: self.theme_revision,
        }
    }
}
