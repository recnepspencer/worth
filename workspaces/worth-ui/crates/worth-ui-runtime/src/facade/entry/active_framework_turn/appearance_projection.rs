use super::WorthUiActiveFrameworkTurnExecution;

impl WorthUiActiveFrameworkTurnExecution<'_> {
    pub(in crate::facade::entry) fn present_prepared_frame_with_appearance(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> crate::mounting::UiMountedPublicationTransition {
        let generation = &self.generation_identity;
        let portal = self.portal.as_deref();
        let motion = self.motion.as_deref();
        let presentation = &*self.presentation;
        let capabilities = self.capabilities;
        let appearance = self.appearance_owner_snapshot.as_ref();
        let overlays = &self.overlay_appearance;
        self.mounted.present_prepared_frame_with_overlays(
            self.host_session,
            frame,
            Some(self.appearance_inspection),
            deadline,
            now,
            |attempt, surfaces, prepared_binding| {
                overlays.as_ref().map_err(|_| ())?.lower(
                    attempt,
                    surfaces,
                    self.overlay_composition_owners,
                    generation,
                    portal,
                    motion,
                    presentation,
                    capabilities,
                    appearance,
                    prepared_binding,
                )
            },
        )
    }

    pub(super) fn appearance_invalidation_for_content(
        &self,
        content: &crate::mounting::UiMountedSemanticContentInput,
        predecessor: Option<&crate::mounting::UiPreparedMountedFrame>,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        let mut invalidation = self.presentation.appearance_invalidation_batch();
        let mounts = crate::runtime::appearance::UiAppearanceInvalidationBatch::mounted_initial(
            self.consumed_facts,
            self.mounted,
            &self.mounted.newly_projected_instances(predecessor),
        );
        if mounts.selected_count() != 0 {
            match invalidation.as_mut() {
                Some(pending) => pending.merge(mounts),
                None => invalidation = Some(mounts),
            }
        }
        let items = self
            .mounted
            .selection_projection_changed_instances(content, predecessor.map(|frame| &**frame));
        if !items.is_empty() {
            let collection =
                crate::runtime::appearance::UiAppearanceInvalidationBatch::mounted_selection_input(
                    self.consumed_facts,
                    self.mounted,
                    &items,
                );
            if collection.selected_count() != 0 {
                match invalidation.as_mut() {
                    Some(pending) => pending.merge(collection),
                    None => invalidation = Some(collection),
                }
            }
        }
        invalidation
    }

    pub(super) fn finish_appearance_projection(
        &mut self,
        mut frame: crate::mounting::UiAssembledMountedFrame,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        frame.stage_pointer_affordance(self.pointer_affordance_snapshot.as_ref(), self.mounted)?;
        self.finish_appearance_projection_from(
            frame,
            self.appearance_owner_snapshot.as_ref(),
            super::super::appearance_projection::UiAppearanceProjectionPhase::Current,
        )
    }

    pub(super) fn finish_reconstruction_appearance_projection(
        &mut self,
        mut frame: crate::mounting::UiAssembledMountedFrame,
    ) -> Result<
        (
            crate::mounting::UiPreparedMountedFrame,
            super::super::mounted_owner_receipt_succession::UiPreparedMountedOwnerReceiptSuccession,
        ),
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        frame.stage_pointer_affordance(self.pointer_affordance_snapshot.as_ref(), self.mounted)?;
        let receipts = self.prepare_mounted_owner_receipts_for_assembled(&frame);
        let snapshot = self.appearance_owner_snapshot.as_ref().and_then(|owners| {
            owners.refresh_receipt_sources(
                self.focus.as_deref(),
                self.selection.as_deref(),
                self.intent_admission,
                self.intent_application_facts,
                self.interaction,
            )
        });
        let frame = self.finish_appearance_projection_from(
            frame,
            snapshot.as_ref(),
            super::super::appearance_projection::UiAppearanceProjectionPhase::Current,
        )?;
        Ok((frame, receipts))
    }

    pub(super) fn finish_retained_appearance_projection(
        &mut self,
        mut frame: crate::mounting::UiAssembledMountedFrame,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
        pointer: &crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        frame.stage_pointer_affordance_succession(pointer, self.mounted)?;
        self.finish_appearance_projection_from(
            frame,
            owners.projection(),
            super::super::appearance_projection::UiAppearanceProjectionPhase::Current,
        )
    }

    pub(super) fn finish_theme_appearance_projection(
        &self,
        mut frame: crate::mounting::UiAssembledMountedFrame,
        theme: &crate::runtime::appearance::UiThemeSwitchChange,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        frame.stage_pointer_affordance(self.pointer_affordance_snapshot.as_ref(), self.mounted)?;
        self.finish_appearance_projection_from(
            frame,
            self.appearance_owner_snapshot.as_ref(),
            super::super::appearance_projection::UiAppearanceProjectionPhase::ThemeSwitch(theme),
        )
    }

    fn finish_appearance_projection_from(
        &self,
        frame: crate::mounting::UiAssembledMountedFrame,
        snapshot: Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
        phase: super::super::appearance_projection::UiAppearanceProjectionPhase<'_>,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        frame.resolve_appearance(
            super::super::appearance_projection::UiAppearanceFrameProjection {
                application_session_identity: self.application_session_identity,
                generation_identity: self.generation_identity.clone(),
                graph: self.graph,
                capabilities: self.capabilities,
                consumed_facts: self.consumed_facts,
                intent_catalog: self.intent_catalog,
                mounted: self.mounted,
                presentation: self.presentation,
                appearance_owner_snapshot: snapshot,
                phase,
            },
        )
    }
}

#[cfg(test)]
#[path = "appearance_projection_locality_tests.rs"]
mod locality_tests;
