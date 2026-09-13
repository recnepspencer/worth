use super::super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn prepare_content_rebind_frame(
        &mut self,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        frame_request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        self.prepare_content_frame(semantic_content, frame_request, None, None)
    }

    pub(in crate::facade::entry) fn prepare_authored_content_frame(
        &mut self,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        frame_request: crate::mounting::UiMountedFrameRequest,
        pointer: &crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
        occurrence_geometry: &crate::mounting::UiMountedOccurrenceGeometryState,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        self.prepare_content_frame(
            semantic_content,
            frame_request,
            Some((pointer, owners, occurrence_geometry)),
            None,
        )
    }

    pub(in crate::facade::entry) fn prepare_closed_owner_content_frame(
        &mut self,
        content: crate::mounting::UiMountedSemanticContentInput,
        overlay_revision: u64,
        overlays: Vec<crate::mounting::UiMountedPortalOverlayProjectionInput>,
        observation: &mut super::super::intent_consequence_observation::WorthUiClosedConsequenceObservation,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        self.queue_closed_owner_invalidation(observation.appearance.as_ref());
        let request = self
            .mounted_frame_request()
            .with_portal_overlays(overlay_revision, overlays);
        self.prepare_content_frame(content, request, None, Some(observation))
    }

    fn prepare_content_frame(
        &mut self,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        frame_request: crate::mounting::UiMountedFrameRequest,
        succession: Option<(
            &crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
            &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
            &crate::mounting::UiMountedOccurrenceGeometryState,
        )>,
        observation: Option<
            &mut super::super::intent_consequence_observation::WorthUiClosedConsequenceObservation,
        >,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let presentation = self.presentation.project().map_err(|denial| {
            crate::runtime::rebind::UiRebindPreparationDenial::ContentMountedPreparation(Box::new(
                denial,
            ))
        })?;
        let frame = {
            let mut completion = self.execute_framework_turn(|_| {}).map_err(|_| {
                crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
            })?;
            if let Some(observation) = observation {
                completion.appearance_owner_snapshot = &mut observation.appearance;
                completion.pointer_affordance_snapshot = &observation.pointer;
            }
            let mut execution = completion.into_execution().map_err(|_| {
                crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
            })?;
            (match succession {
                Some((pointer, owners, occurrence_geometry)) => execution
                    .prepare_mounted_authored_content(
                        frame_request,
                        semantic_content,
                        presentation,
                        pointer,
                        owners,
                        occurrence_geometry,
                    ),
                None => execution.prepare_mounted_frame_with_content_internal(
                    frame_request,
                    semantic_content,
                    presentation,
                ),
            })
            .map_err(|denial| {
                crate::runtime::rebind::UiRebindPreparationDenial::ContentMountedPreparation(
                    Box::new(denial),
                )
            })?
        };
        Ok(frame)
    }
}

impl<'session> super::WorthUiPreparedMountedContentRebind<'session> {
    pub(in crate::facade::entry) fn prepare(
        session: &'session mut WorthUiActiveApplicationSession,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
    ) -> Result<Self, crate::runtime::rebind::UiRebindPreparationDenial> {
        let frame = session
            .prepare_content_rebind_frame(semantic_content, session.mounted_frame_request())?;
        Ok(Self::new(
            session,
            frame,
            super::WorthUiMountedContentPublication::RetainedGeneration,
        ))
    }
}

impl super::WorthUiPreparedMountedContentRebind<'_> {
    pub(crate) fn refresh_before_effects(
        &mut self,
        plan: &mut crate::runtime::rebind::UiRebindPlan,
    ) -> Result<(), crate::runtime::rebind::UiRebindPreparationDenial> {
        let super::WorthUiMountedContentPublication::AuthoredSuccessor {
            authority,
            pointer_succession,
            appearance_succession,
            occurrence_geometry,
            owners,
            ..
        } = &mut self.publication
        else {
            return Ok(());
        };
        if pointer_succession
            .validate_predecessor(
                &self.session.active_generation_identity(),
                self.session.pointer_affordance_snapshot.as_ref(),
                self.session
                    .observation_clock
                    .as_ref()
                    .map(|clock| clock.sample_millis()),
                &self.session.mounted,
            )
            .is_ok()
            && owners.matches_projection(self.session.appearance_owner_snapshot.as_ref())
            && self
                .session
                .prepare_retained_appearance_owners(authority, appearance_succession)
                .is_ok()
        {
            self.session
                .include_pointer_publication(plan, pointer_succession)?;
            return Ok(());
        }
        let refreshed = self
            .session
            .prepare_pointer_generation_succession(authority)?;
        let refreshed_owners = self
            .session
            .prepare_retained_appearance_owners(authority, appearance_succession)?;
        self.session.include_pointer_publication(plan, &refreshed)?;
        let frame = self.session.prepare_authored_content_frame(
            plan.content().clone(),
            self.session.mounted_frame_request(),
            &refreshed,
            &refreshed_owners,
            occurrence_geometry,
        )?;
        *pointer_succession = refreshed;
        *owners = refreshed_owners;
        self.frame = frame;
        Ok(())
    }
}
