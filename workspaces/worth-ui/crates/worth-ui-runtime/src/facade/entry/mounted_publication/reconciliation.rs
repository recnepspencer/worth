use super::super::WorthUiActiveApplicationSession;
use crate::mounting::UiMountedFrameOutcome;

impl WorthUiActiveApplicationSession {
    pub fn present_current_mounted_frame_for_reconciliation(
        &mut self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, crate::mounting::UiMountedIdentityDenial> {
        let overlays = self.prepare_overlay_appearance_sources();
        let generation = self.generation_identity().clone();
        let portal = self.portal.as_ref();
        let motion = self.motion.as_ref();
        let presentation = &self.presentation;
        let capabilities = self.application.capabilities();
        let appearance = self.appearance_owner_snapshot.as_ref();
        let transition = self.mounted.present_current_for_reconciliation(
            &self.host_session,
            replacements,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
            |attempt, surfaces| {
                overlays.as_ref().map_err(|_| ())?.lower(
                    attempt,
                    surfaces,
                    &mut self.overlay_composition_owners,
                    &generation,
                    portal,
                    motion,
                    presentation,
                    capabilities,
                    appearance,
                )
            },
        )?;
        Ok(self.finish_mounted_transition(transition))
    }

    pub(crate) fn present_prepared_mounted_frame_for_reconciliation(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, crate::mounting::UiMountedIdentityDenial> {
        let overlays = self.prepare_overlay_appearance_sources();
        let generation = self.generation_identity().clone();
        let portal = self.portal.as_ref();
        let motion = self.motion.as_ref();
        let presentation = &self.presentation;
        let capabilities = self.application.capabilities();
        let appearance = self.appearance_owner_snapshot.as_ref();
        let transition = self.mounted.present_prepared_for_reconciliation(
            &self.host_session,
            frame,
            replacements,
            Some(&mut self.appearance_inspection),
            deadline,
            now,
            |attempt, surfaces| {
                overlays.as_ref().map_err(|_| ())?.lower(
                    attempt,
                    surfaces,
                    &mut self.overlay_composition_owners,
                    &generation,
                    portal,
                    motion,
                    presentation,
                    capabilities,
                    appearance,
                )
            },
        )?;
        Ok(self.finish_mounted_transition(transition))
    }
}
