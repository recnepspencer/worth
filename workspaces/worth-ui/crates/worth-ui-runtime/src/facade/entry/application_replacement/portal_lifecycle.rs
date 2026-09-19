/// Prepared Portal/Motion replacement work. This bundle is created before a
/// replacement frame can reach the host and is consumed only after that frame
/// has published, so Portal membership never races service reconciliation.
pub(super) struct WorthUiPreparedApplicationLifecycle {
    pub(super) overlay_bindings: crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    scroll: super::scroll_replacement::UiPreparedScrollReplacement,
    motion_rebind: Option<crate::runtime::motion::UiPreparedMotionRebind>,
    portal_removal: Option<crate::runtime::portal::UiPreparedPortalRebindRemoval>,
    retained_exit_retentions: Box<
        [(
            crate::runtime::portal::UiPortalIdentity,
            crate::runtime::motion::UiMotionExitRetentionReceipt,
        )],
    >,
}

impl WorthUiPreparedApplicationLifecycle {
    pub(super) fn geometry_validation_inputs(
        &mut self,
    ) -> (
        &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
        Option<&mut crate::runtime::scroll::UiScrollRuntimeState>,
    ) {
        (&self.overlay_bindings, self.scroll.state_mut())
    }

    pub(super) fn take_staged_scroll(
        &mut self,
    ) -> super::scroll_replacement::UiPreparedScrollReplacement {
        std::mem::replace(
            &mut self.scroll,
            super::scroll_replacement::UiPreparedScrollReplacement::empty(),
        )
    }
}

impl super::WorthUiActiveApplicationSession {
    pub(super) fn prepare_application_lifecycle(
        &self,
        successor: &crate::mounting::UiMountedGraphReplacementSuccessor,
        application: &super::WorthUiPreparedApplicationActivation,
    ) -> Result<WorthUiPreparedApplicationLifecycle, super::WorthUiApplicationCutoverDenial> {
        let portal_policy = application.candidate_service_policy_plan().portal();
        let portal_removal = self.portal.as_ref().map(|portal| {
            portal
                .prepare_rebound_portal_removal(&successor.identity_view(), portal_policy.is_none())
        });
        let removed_portals = portal_removal
            .as_ref()
            .map(|removal| removal.removed())
            .unwrap_or(&[]);
        let overlay_bindings = self
            .authored_overlay_bindings
            .prepare_application_replacement(
                self.application.prepared_authority(),
                application.candidate_replacement_authority(),
                removed_portals,
            )
            .map_err(|_| {
                super::WorthUiApplicationCutoverDenial::OverlayBindingSuccessionUnavailable
            })?;
        Ok(WorthUiPreparedApplicationLifecycle {
            overlay_bindings,
            scroll: self.prepare_scroll_replacement_state(application),
            motion_rebind: self
                .motion
                .as_ref()
                .map(|motion| motion.prepare_mounted_rebind(successor)),
            retained_exit_retentions: self
                .portal_exit_retention
                .retentions_for_portals(removed_portals),
            portal_removal,
        })
    }

    pub(super) fn commit_application_lifecycle(
        &mut self,
        mut lifecycle: WorthUiPreparedApplicationLifecycle,
    ) {
        if let Some(removal) = lifecycle.portal_removal.as_ref() {
            self.portal
                .as_ref()
                .expect("a prepared Portal removal retains its Portal owner")
                .validate_rebound_portal_removal(removal)
                .expect("a replacement lifecycle retains the current Portal revision");
        }
        self.authored_overlay_bindings = lifecycle.overlay_bindings;
        let rebound_terminals = match (self.motion.as_mut(), lifecycle.motion_rebind.take()) {
            (Some(motion), Some(prepared)) => motion.commit_mounted_rebind(prepared),
            _ => Box::default(),
        };
        let rebound_tracks: Vec<_> = rebound_terminals
            .iter()
            .filter_map(|terminal| terminal.exit_retention().map(|retention| retention.track()))
            .collect();
        for terminal in rebound_terminals {
            self.release_rebound_motion_retention(terminal);
            self.retire_rebound_motion_sample(terminal.track());
        }

        for (portal, motion) in lifecycle.retained_exit_retentions {
            if rebound_tracks.contains(&motion.track()) {
                continue;
            }
            assert!(
                self.portal_exit_retention.has_portal(portal),
                "prepared Portal exit retention lost its coordinator before commit"
            );
            let terminal = self
                .motion
                .as_mut()
                .expect("rebound Portal exit retention retains Motion installation")
                .terminalize(
                    motion.track(),
                    crate::runtime::motion::UiMotionTerminalCause::ReboundAway,
                );
            if let Some(terminal) = terminal {
                assert_eq!(
                    terminal.exit_retention().map(|retention| retention.track()),
                    Some(motion.track()),
                    "rebound Motion terminal must retain its prepared track"
                );
            }
            self.release_rebound_portal_retention(portal);
        }

        if let Some(removal) = lifecycle.portal_removal {
            self.portal
                .as_mut()
                .expect("a prepared Portal removal retains its Portal owner")
                .commit_rebound_portal_removal(removal)
                .expect("a replacement lifecycle retains the current Portal revision");
        }
    }
}
