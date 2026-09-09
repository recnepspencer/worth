use super::session::World;
use crate::runtime::portal::*;
use worth_ui_host_contract::*;

impl World {
    pub(super) fn sample(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        tick: u64,
        epoch: u64,
        expected_opacity: u16,
    ) {
        let session = &mut self.session;
        let basis = session
            .mounted
            .current_presentation_for_surface(surface)
            .unwrap();
        let prepared = session.prepare_motion_tick(tick, basis).unwrap();
        assert_eq!(prepared.receipt().samples().len(), 3);
        for sample in prepared.receipt().samples() {
            assert_eq!(sample.opacity_units(), expected_opacity);
        }
        self.host
            .push_presentation(UiHostSurfacePresentationOutcome::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(epoch),
                    UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                    UiHostPresentationCostReport::default(),
                ),
            ));
        session.present_prepared_motion_tick(prepared, basis);
        assert_eq!(
            session
                .inspect_motion_presentation_for_certification()
                .opacity_units(),
            Some(expected_opacity)
        );
    }

    pub(super) fn close_parent(
        &mut self,
        parent: UiPortalIdentity,
        child: UiPortalIdentity,
        surviving_sibling: UiPortalIdentity,
        restoration_target: UiMountedInstanceIdentity,
        now: u64,
    ) {
        for _ in self.surfaces {
            self.host.push_native_display_settled_without_effects();
        }
        let focus = match self
            .session
            .publish_anchor_loss_portal_dismissal(parent, now)
        {
            crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome::Published(
                receipt,
            ) => receipt.focus_publication(),
            _ => panic!("parent Portal dismissal must publish"),
        };
        assert_eq!(
            focus.cause(),
            crate::facade::entry::UiSemanticFocusPublicationCause::PortalRestoration
        );
        assert_eq!(
            focus.outcome(),
            crate::facade::entry::UiSemanticFocusPublicationOutcome::Moved
        );
        assert_eq!(
            focus.previous().unwrap().mounted_instance(),
            self.instances[2],
            "the nested Portal owner held Focus before its ancestor closed"
        );
        assert_eq!(
            focus.current().unwrap().mounted_instance(),
            restoration_target,
            "dismissal restores the exact pre-Portal mounted occurrence"
        );
        let portal = self.session.portal.as_ref().unwrap();
        assert_eq!(portal.posture(parent), UiPortalLifecyclePosture::Closing);
        assert_eq!(portal.posture(child), UiPortalLifecyclePosture::Closed);
        assert_eq!(
            portal.posture(surviving_sibling),
            UiPortalLifecyclePosture::Visible
        );
        assert_eq!(portal.current_mounted_projection_inputs().len(), 2);

        let basis = self
            .session
            .mounted
            .current_presentation_for_surface(self.surfaces[0])
            .unwrap();
        let prepared = self.session.prepare_motion_tick(now + 1, basis).unwrap();
        assert_eq!(prepared.receipt().samples().len(), 1);
        assert_eq!(prepared.receipt().samples()[0].opacity_units(), 65_535);
        self.host
            .push_presentation(UiHostSurfacePresentationOutcome::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(90),
                    UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                    UiHostPresentationCostReport::default(),
                ),
            ));
        self.session.present_prepared_motion_tick(prepared, basis);
        assert!(!self.session.portal_exit_terminal_work_pending());

        let basis = self
            .session
            .mounted
            .current_presentation_for_surface(self.surfaces[0])
            .unwrap();
        let prepared = self.session.prepare_motion_tick(now + 56, basis).unwrap();
        assert_eq!(prepared.receipt().samples().len(), 1);
        assert_eq!(prepared.receipt().samples()[0].opacity_units(), 8_192);
        self.host
            .push_presentation(UiHostSurfacePresentationOutcome::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(91),
                    UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                    UiHostPresentationCostReport::default(),
                ),
            ));
        self.session.present_prepared_motion_tick(prepared, basis);
        assert!(!self.session.portal_exit_terminal_work_pending());
        let composed = self.prepare_surface_with_current_portals(self.surfaces[0]);
        self.publish(composed, now + 57, false);
        super::motion_reconstruction::reject_retry_at_nonterminal_sample(
            self,
            8_192,
            now + 58,
            now + 59,
        );

        let basis = self
            .session
            .mounted
            .current_presentation_for_surface(self.surfaces[0])
            .unwrap();
        let prepared = self.session.prepare_motion_tick(now + 112, basis).unwrap();
        assert_eq!(prepared.receipt().samples().len(), 1);
        assert_eq!(prepared.receipt().samples()[0].opacity_units(), 0);
        self.host
            .push_presentation(UiHostSurfacePresentationOutcome::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(92),
                    UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                    UiHostPresentationCostReport::default(),
                ),
            ));
        self.session.present_prepared_motion_tick(prepared, basis);
        assert!(self.session.portal_exit_terminal_work_pending());

        for (epoch, effects) in [
            (93, vec![UiMountedEffectFamily::NativePaint]),
            (94, Vec::new()),
        ] {
            self.host
                .push_presentation(UiHostSurfacePresentationOutcome::Presented(
                    UiMountedSurfacePresentationCompletion::new(
                        UiHostSurfacePresentationMode::NativeDisplay,
                        UiHostPresentationEpoch::issued_by_host(epoch),
                        UiMountedCompletedEffects::new(effects),
                        UiHostPresentationCostReport::default(),
                    ),
                ));
        }
        let progress = self.session.progress_portal_exit_terminal(now + 113);
        assert_eq!(
            progress,
            crate::facade::entry::active_application_session::UiPortalExitTerminalProgress::Published
        );
        let portal = self.session.portal.as_ref().unwrap();
        assert_eq!(portal.posture(parent), UiPortalLifecyclePosture::Closed);
        assert_eq!(portal.posture(child), UiPortalLifecyclePosture::Closed);
        assert_eq!(
            portal.posture(surviving_sibling),
            UiPortalLifecyclePosture::Visible
        );
        assert_eq!(portal.current_mounted_projection_inputs().len(), 1);

        let output = self
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .unwrap();
        let overlay = output
            .fragments()
            .iter()
            .find(|fragment| {
                fragment.identity()
                    == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(self.surfaces[0])
            })
            .expect("terminal parent closure publishes overlay retirement");
        assert!(overlay
            .work()
            .successor()
            .mechanics()
            .iter()
            .all(|mechanic| {
                match mechanic {
                    UiMountedAppearanceMechanic::Backdrop(backdrop) => {
                        backdrop.identity().scope()
                            != UiMountedBackdropScope::PerPortalInstance(
                                parent.owner().mounted_instance_identity(),
                            )
                    }
                    UiMountedAppearanceMechanic::PortalSurface(surface) => {
                        surface.portal_instance() != parent.owner().mounted_instance_identity()
                            && surface.portal_instance()
                                != child.owner().mounted_instance_identity()
                    }
                    _ => true,
                }
            }));
    }
}
