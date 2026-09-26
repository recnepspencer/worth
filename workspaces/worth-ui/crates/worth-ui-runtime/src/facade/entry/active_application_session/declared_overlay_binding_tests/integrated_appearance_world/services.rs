use super::session::World;
use crate::runtime::portal::*;
use worth_ui_host_contract::*;

/// Where an open finds its anchor, and what extent the Portal takes.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Opening {
    /// From the owner's launched box, at the declared extent.
    Launched,
    /// From wherever the owner presents now, at the declared extent.
    WherePresented,
    /// From the owner's launched box, fitted to the content it lays out, as
    /// an activated open is.
    FittedToContent,
}

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
        self.open_inspecting_frame(index, declaration, parent, now, |_| {})
    }

    pub(super) fn open_inspecting_frame(
        &mut self,
        index: usize,
        declaration: &str,
        parent: Option<UiPortalIdentity>,
        now: u64,
        inspect: impl FnOnce(&crate::mounting::UiPreparedMountedFrame),
    ) -> UiPortalIdentity {
        self.open_from(index, declaration, parent, now, Opening::Launched, inspect)
    }

    /// Opens a top-level Portal fitted to its content, as activation does.
    pub(super) fn open_fitted(
        &mut self,
        index: usize,
        declaration: &str,
        now: u64,
    ) -> UiPortalIdentity {
        self.open_from(
            index,
            declaration,
            None,
            now,
            Opening::FittedToContent,
            |_| {},
        )
    }

    /// Opens from wherever the owner is presented now, which for Portal
    /// content or scrolled content is not its launched box.
    pub(super) fn open_where_presented(
        &mut self,
        index: usize,
        declaration: &str,
        parent: Option<UiPortalIdentity>,
        now: u64,
    ) -> UiPortalIdentity {
        self.open_from(
            index,
            declaration,
            parent,
            now,
            Opening::WherePresented,
            |_| {},
        )
    }

    /// The graph node each mounted instance occurs for.
    pub(super) fn owner_graph(&self, index: usize) -> crate::graph::UiGraphNodeIdentity {
        self.graphs[[0, 1, 2, 0, 3][index]]
    }

    fn open_from(
        &mut self,
        index: usize,
        declaration: &str,
        parent: Option<UiPortalIdentity>,
        now: u64,
        opening: Opening,
        inspect: impl FnOnce(&crate::mounting::UiPreparedMountedFrame),
    ) -> UiPortalIdentity {
        let owner_graph = self.owner_graph(index);
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
            owner_graph,
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
            .interaction_hit_test_basis(presentation.basis())
            .unwrap()
            .rows()
            .iter()
            .find(|row| row.mounted_instance() == instance)
            .unwrap();
        assert_eq!(
            row.bounds().coordinate_space(),
            UiMountedCoordinateSpace::Viewport
        );
        let presented = row.bounds().platform_box();
        let [x, y, width, height] = [
            presented.x(),
            presented.y(),
            presented.width(),
            presented.height(),
        ];
        if opening != Opening::WherePresented {
            assert_eq!([x, y, width, height], super::geometry::BOXES[index]);
        }
        assert_eq!(
            session.mounted.current_surface_viewport(surface).unwrap().1,
            super::geometry::canonical(super::geometry::VIEWPORT)
        );
        // Aim at the middle of what the frame shows of the row: the regions it
        // sits inside can clip it, and a pointer reaches only what they leave.
        let shown = row
            .reachable_box()
            .expect("the frame shows the row an overlay opens from");
        let target = crate::runtime::interaction::targeting::resolve_presented_target(
            &session.mounted,
            presentation.basis(),
            UiHostSurfacePosition::viewport_logical(
                ((shown.x() + shown.width() / 2.0) * 1_000.0) as i64,
                ((shown.y() + shown.height() / 2.0) * 1_000.0) as i64,
            ),
            &mut Default::default(),
        )
        .unwrap();
        assert_eq!(target.mounted_instance(), instance);
        let anchor = target.view().geometry();
        let committed_viewport = session.mounted.current_portal_viewport(surface).unwrap();
        let viewport =
            crate::runtime::interaction::UiPresentedViewportGeometry::from_current_interaction(
                committed_viewport,
                anchor,
            )
            .unwrap();
        assert_eq!(
            viewport.bounds().canonical_box(),
            super::geometry::viewport(super::geometry::VIEWPORT)
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
        let request = match opening {
            Opening::FittedToContent => request.with_content_extent(
                session
                    .mounted
                    .current_portal_content_extent(instance)
                    .unwrap(),
            ),
            Opening::Launched | Opening::WherePresented => request,
        };
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
                presentation.basis(),
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
        inspect(&frame);
        self.host.push_native_display_presented();
        match crate::facade::entry::portal_dismissal::present_portal_service_proposal(session, frame, proposal, false, now) {
            crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome::Published(_) => {},
            crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome::Stopped(stop) => panic!("declared Portal {index} proposal: {stop:?}"),
            other => panic!("declared Portal {index} proposal: {:?}", std::mem::discriminant(&other)),
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
