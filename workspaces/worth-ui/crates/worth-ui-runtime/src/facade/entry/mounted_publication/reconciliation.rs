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
            |attempt, surfaces, prepared_binding| {
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
                    prepared_binding,
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
        let owner_receipts = self.prepare_mounted_owner_receipt_succession(&frame);
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
            |attempt, surfaces, prepared_binding| {
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
                    prepared_binding,
                )
            },
        )?;
        let outcome = self.finish_mounted_transition(transition);
        self.settle_new_mounted_owner_receipt_succession(owner_receipts, &outcome);
        Ok(outcome)
    }

    pub(crate) fn present_prepared_mounted_reconstruction_frame(
        &mut self,
        prepared: super::super::mounted_frame_execution::UiPreparedMountedReconstructionFrame,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedFrameOutcome, crate::mounting::UiMountedIdentityDenial> {
        let (frame, owner_receipts) = prepared.into_parts();
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
            |attempt, surfaces, prepared_binding| {
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
                    prepared_binding,
                )
            },
        )?;
        let outcome = self.finish_mounted_transition(transition);
        self.settle_new_mounted_owner_receipt_succession(owner_receipts, &outcome);
        Ok(outcome)
    }
}
