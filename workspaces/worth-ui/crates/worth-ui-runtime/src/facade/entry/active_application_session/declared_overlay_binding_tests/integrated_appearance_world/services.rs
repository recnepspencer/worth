use super::session::World;
use crate::runtime::portal::*;
use worth_ui_host_contract::*;

impl World {
    pub(super) fn focus_first_surface_participant(&mut self) -> UiMountedInstanceIdentity {
        let focus = self.session.focus.as_mut().unwrap();
        let scope = focus.default_scope_for_surface(self.surfaces[0]).unwrap();
        let transition = focus
            .commit_host_traversal(
                scope,
                crate::runtime::focus::UiHostFocusTraversalDirection::Forward,
                false,
            )
            .unwrap();
        let mounted_instance = transition.current().unwrap().mounted_instance();
        assert_eq!(mounted_instance, self.instances[0]);
        mounted_instance
    }

    // The request substitutes for external activation. Declared admission, service
    // proposals, mounted publication, binding/Motion settlement and sampling are real.
    pub(super) fn open(
        &mut self,
        index: usize,
        declaration: &str,
        parent: Option<UiPortalIdentity>,
        now: u64,
    ) -> UiPortalIdentity {
        let session = &mut self.session;
        let expected_shielding = if declaration == "overlay.child" {
            UiPortalInputShielding::ModalSurface
        } else {
            UiPortalInputShielding::ContentBounds
        };
        let instance = self.instances[index];
        let surface = if index == 3 {
            self.surfaces[1]
        } else {
            self.surfaces[0]
        };
        let portal = UiPortalIdentity::for_owner(UiPortalOwnerIdentity::from_mounted_owner(
            self.graphs[if index == 3 { 0 } else { index }],
            instance,
        ));
        let declaration = session
            .application
            .authored_overlay_material()
            .overlay_declaration_bindings()
            .portal_named(declaration)
            .unwrap();
        let presentation = session
            .mounted
            .current_presentation_for_surface(surface)
            .unwrap();
        let row = *session
            .mounted
            .interaction_hit_test_basis(presentation)
            .unwrap()
            .rows()
            .iter()
            .find(|row| row.mounted_instance() == instance)
            .unwrap();
        assert_eq!(
            row.bounds().coordinate_space(),
            UiMountedCoordinateSpace::Viewport
        );
        assert_eq!(
            [
                row.bounds().x(),
                row.bounds().y(),
                row.bounds().width(),
                row.bounds().height()
            ],
            super::geometry::BOXES[index]
        );
        assert_eq!(
            session.mounted.current_surface_viewport(surface).unwrap().1,
            super::geometry::canonical(super::geometry::VIEWPORT)
        );
        let anchor =
            crate::runtime::interaction::UiPresentedInteractionGeometry::for_test_with_components(
                presentation,
                super::geometry::BOXES[index],
                super::geometry::VIEWPORT,
            );
        let viewport = crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
            anchor.clip_bounds(),
            presentation,
        );
        let idempotency =
            crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
                session.session_identity().as_u64(),
                now,
            );
        let request = match parent {
            Some(parent) => UiPortalServiceRequest::open_nested(
                portal,
                idempotency,
                anchor,
                viewport,
                surface,
                parent,
                UiPortalInputShielding::ContentBounds,
            ),
            None => {
                UiPortalServiceRequest::open(portal, idempotency, anchor, Some(viewport), surface)
            }
        }
        .with_declared_portal(Some(declaration));
        let binding = session
            .admit_authored_portal_open(declaration, portal, surface)
            .unwrap();
        let policy = session.authored_portal_policy(declaration).unwrap();
        let transition = session
            .portal
            .as_ref()
            .unwrap()
            .prepare_authored(request, policy)
            .unwrap();
        assert_eq!(
            transition.placement().unwrap().shielding(),
            expected_shielding,
            "the exact authored Portal declaration owns input shielding"
        );
        let motion = session.prepare_portal_motion_request(&transition).unwrap();
        assert!(
            motion.is_some(),
            "authored Portal produces its Motion transition"
        );
        let revision = transition.successor_revision();
        let overlays = session
            .portal
            .as_ref()
            .unwrap()
            .mounted_projection_inputs(&transition, false);
        let generation = session.active_generation_identity();
        let preparation = session
            .application
            .begin_portal_service_proposal_for_certification(
                transition,
                presentation,
                generation,
                Some(binding),
                session.motion.as_mut().unwrap(),
                motion,
            )
            .unwrap();
        let frame = session
            .prepare_intent_consequence_frame_for_surface_for_test(
                crate::mounting::UiMountedSemanticContentInput::empty(),
                revision,
                overlays,
                surface,
            )
            .unwrap();
        let scroll_incarnation = session.scroll_owner_incarnation();
        let proposal = session
            .application
            .bind_portal_service_proposal_frame(
                preparation,
                &frame,
                &session.mounted,
                session.focus.as_mut().unwrap(),
                session.scroll.as_ref(),
                scroll_incarnation,
                session.motion.as_mut().unwrap(),
            )
            .unwrap();
        self.host.push_native_display_presented();
        let outcome = session.present_prepared_portal_frame_internal(
            frame,
            &proposal,
            false,
            UiPresentationDeadline::at_tick(u64::MAX),
            now,
        );
        let diagnostic = match &outcome {
            crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                format!("{:?}", frame.report())
            }
            crate::mounting::UiMountedFrameOutcome::AdmissionDenied(denial) => {
                format!("{:?}", denial.denial())
            }
            _ => format!("{:?}", std::mem::discriminant(&outcome)),
        };
        match crate::facade::entry::portal_dismissal::finish_portal_service_proposal(session, proposal, outcome) {
            crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome::Published(_) => {},
            crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome::Stopped(stop) => panic!("declared Portal {index} proposal: {stop:?}; {diagnostic}"),
            other => panic!("declared Portal {index} proposal: {:?}; {diagnostic}", std::mem::discriminant(&other)),
        }
        assert_eq!(
            session
                .portal
                .as_ref()
                .unwrap()
                .placement(portal)
                .unwrap()
                .prepared()
                .layer()
                .parent(),
            parent
        );
        portal
    }
}
