use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn publish_nested_portal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> crate::certification_support::UiPortalNestedCertificationOutcome {
        let Some((parent, presentation)) = self
            .portal
            .as_ref()
            .and_then(|portal| portal.stack_snapshot().rows().last().copied())
            .and_then(|row| {
                self.portal
                    .as_ref()
                    .and_then(|portal| portal.committed_presentation_for(row.portal()))
                    .map(|presentation| (row.portal(), presentation))
            })
        else {
            return crate::certification_support::UiPortalNestedCertificationOutcome::NotPublished;
        };
        let Some(child) = self
            .mounted
            .view()
            .mounted_instances()
            .iter()
            .map(|instance| instance.identity())
            .find(|instance| *instance != parent.owner().mounted_instance_identity())
        else {
            return crate::certification_support::UiPortalNestedCertificationOutcome::NotPublished;
        };
        let Some(child_basis) = self.mounted.current_mounted_identity_basis(child) else {
            return crate::certification_support::UiPortalNestedCertificationOutcome::NotPublished;
        };
        let child_portal = crate::runtime::portal::UiPortalIdentity::for_owner(
            crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(
                child_basis.graph_node_identity(),
                child,
            ),
        );
        let anchor =
            crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation);
        let request = crate::runtime::portal::UiPortalServiceRequest::open_nested(
            child_portal,
            self.next_portal_service_idempotency(),
            anchor,
            crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
                anchor.clip_bounds(),
                presentation,
            ),
            child_basis.semantic_surface_identity(),
            parent,
            crate::runtime::portal::UiPortalInputShielding::ContentBounds,
        );
        let transition = match self
            .portal
            .as_ref()
            .expect("nested Portal certification requires Portal installation")
            .prepare(request)
        {
            Ok(transition) => transition,
            Err(denial) => panic!("nested Portal request was denied: {denial:?}"),
        };
        let motion_request = match self.prepare_portal_motion_request(&transition) {
            Ok(motion_request) => motion_request,
            Err(denial) => panic!("nested Portal Motion request was denied: {denial:?}"),
        };
        let revision = transition.successor_revision();
        let overlays = self
            .portal
            .as_ref()
            .expect("nested Portal certification retains Portal installation")
            .mounted_projection_inputs(&transition, transition.closes_portal());
        let preparation = match self
            .application
            .begin_portal_service_proposal_for_certification(
                transition,
                presentation,
                self.active_generation_identity(),
                None,
                self.motion
                    .as_mut()
                    .expect("nested Portal certification retains Motion installation"),
                motion_request,
            ) {
            Ok(preparation) => preparation,
            Err(denial) => panic!("nested Portal proposal was denied: {denial:?}"),
        };
        let frame = match self.prepare_intent_consequence_frame(
            crate::mounting::UiMountedSemanticContentInput::empty(),
            revision,
            overlays,
        ) {
            Ok(frame) => frame,
            Err(denial) => {
                self.application.cancel_portal_service_proposal_preparation(
                    preparation,
                    self.motion
                        .as_mut()
                        .expect("nested Portal certification retains Motion installation"),
                );
                panic!("nested Portal frame was denied: {denial:?}");
            }
        };
        let scroll_incarnation = self.scroll_owner_incarnation();
        let proposal = match self.application.bind_portal_service_proposal_frame(
            preparation,
            &frame,
            &self.mounted,
            self.focus
                .as_mut()
                .expect("nested Portal certification retains Focus installation"),
            self.scroll.as_ref(),
            scroll_incarnation,
            self.motion
                .as_mut()
                .expect("nested Portal certification retains Motion installation"),
        ) {
            Ok(proposal) => proposal,
            Err(denial) => panic!("nested Portal proposal binding was denied: {denial:?}"),
        };
        let outcome = self.present_prepared_portal_frame_internal(
            frame,
            &proposal,
            false,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
            now_tick,
        );
        match super::portal_dismissal::finish_portal_service_proposal(self, proposal, outcome) {
            super::portal_dismissal::UiPortalDismissalPublicationOutcome::Published(_) => {
                crate::certification_support::UiPortalNestedCertificationOutcome::Published
            }
            super::portal_dismissal::UiPortalDismissalPublicationOutcome::InFlight(completion) => {
                drop(completion);
                crate::certification_support::UiPortalNestedCertificationOutcome::NotPublished
            }
            super::portal_dismissal::UiPortalDismissalPublicationOutcome::Indeterminate(
                recovery,
            ) => {
                drop(recovery);
                crate::certification_support::UiPortalNestedCertificationOutcome::NotPublished
            }
            super::portal_dismissal::UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal
            | super::portal_dismissal::UiPortalDismissalPublicationOutcome::IgnoredInsideTopmostPortal
            | super::portal_dismissal::UiPortalDismissalPublicationOutcome::Stopped(_) => {
                crate::certification_support::UiPortalNestedCertificationOutcome::NotPublished
            }
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn publish_root_portal_dismissal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> super::portal_dismissal::UiPortalDismissalPublicationOutcome<'_> {
        let root = self
            .portal
            .as_ref()
            .and_then(|portal| portal.stack_snapshot().rows().first().copied())
            .map(|row| row.portal());
        root.map_or(
            super::portal_dismissal::UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal,
            |root| self.publish_anchor_loss_portal_dismissal(root, now_tick),
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    fn next_portal_service_idempotency(
        &mut self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
        let lineage = self.next_portal_service_event_identity;
        self.next_portal_service_event_identity = lineage
            .checked_add(1)
            .expect("certification Portal event identity remains representable");
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            self.session_identity().as_u64(),
            lineage,
        )
    }
}
