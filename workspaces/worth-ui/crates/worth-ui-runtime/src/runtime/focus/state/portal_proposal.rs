impl super::UiFocusRuntimeState {
    pub(in crate::runtime) fn stage_portal_proposal(
        &mut self,
        owner: &crate::runtime::focus::UiStagedFocusServiceProposal,
        snapshot: crate::mounting::UiMountedFocusParticipationSnapshot,
    ) -> Result<(), crate::runtime::focus::UiPortalFocusTransitionDenial> {
        if self.pending_portal.contains_key(&owner.proposal()) {
            return Err(crate::runtime::focus::UiPortalFocusTransitionDenial::DuplicateProposal);
        }
        let requirement = owner.requirement();
        let ordered = crate::runtime::focus::rebind::focusable_participants(&snapshot);
        let restoration = self
            .policy
            .restores_on_scope_close()
            .then(|| self.restoration_token())
            .flatten();
        let next = if requirement.opening() {
            initial_portal_target(&ordered, &self.participant_index, requirement.owner())
        } else {
            let retained = self
                .portal_restorations
                .get(&requirement.boundary())
                .copied();
            match retained.filter(|_| self.policy.restores_on_scope_close()) {
                Some(restoration) => restoration_target(&ordered, restoration),
                None => current_target(&ordered, self.current),
            }
        };
        let structural_revision = self
            .structural_revision
            .checked_add(1)
            .ok_or(crate::runtime::focus::UiPortalFocusTransitionDenial::Routing)?;
        self.pending_portal.insert(
            owner.proposal(),
            crate::runtime::focus::portal_transition::UiPreparedPortalFocusTransition::new(
                requirement.boundary(),
                requirement.opening(),
                self.revision,
                snapshot,
                next.map(crate::runtime::focus::UiSemanticKeyboardFocus::new),
                restoration,
                requirement.closed_descendants().into(),
            ),
        );
        self.structural_revision = structural_revision;
        Ok(())
    }

    pub(in crate::runtime) fn validate_portal_proposal(
        &self,
        proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) -> Result<(), crate::runtime::focus::UiPortalFocusTransitionDenial> {
        let transition = self
            .pending_portal
            .get(&proposal)
            .ok_or(crate::runtime::focus::UiPortalFocusTransitionDenial::UnknownProposal)?;
        if transition.frame() != frame {
            return Err(crate::runtime::focus::UiPortalFocusTransitionDenial::ForeignPreparedFrame);
        }
        if transition.expected_revision() != self.revision {
            return Err(crate::runtime::focus::UiPortalFocusTransitionDenial::StaleFocusRevision);
        }
        Ok(())
    }

    pub(in crate::runtime) fn staged_portal_reveal_requirement(
        &self,
        proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
    ) -> Result<
        Option<crate::runtime::session::service_proposal::UiFocusRevealRequirement>,
        crate::runtime::focus::UiPortalFocusTransitionDenial,
    > {
        let transition = self
            .pending_portal
            .get(&proposal)
            .ok_or(crate::runtime::focus::UiPortalFocusTransitionDenial::UnknownProposal)?;
        Ok(self
            .policy
            .reveals_focused_target()
            .then(|| transition.next().map(|focus| focus.reveal_requirement()))
            .flatten())
    }

    pub(in crate::runtime) fn commit_portal_proposal(
        &mut self,
        proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) -> Result<
        crate::runtime::focus::UiFocusTransitionReceipt,
        crate::runtime::focus::UiPortalFocusTransitionDenial,
    > {
        self.validate_portal_proposal(proposal, frame)?;
        let transition = self
            .pending_portal
            .remove(&proposal)
            .expect("validated Focus proposal remains staged");
        self.install_mounted_participation(transition.snapshot())
            .map_err(|_| crate::runtime::focus::UiPortalFocusTransitionDenial::Routing)?;
        if transition.opening() {
            self.portal_restorations
                .insert(transition.boundary(), transition.restoration());
        } else {
            self.portal_restorations.remove(&transition.boundary());
            for descendant in transition.closed_descendants() {
                self.portal_restorations.remove(descendant);
            }
        }
        self.apply_immediate(
            transition.next(),
            if transition.opening() {
                crate::runtime::focus::UiFocusCause::PortalInitial
            } else {
                crate::runtime::focus::UiFocusCause::PortalRestoration
            },
            u32::from(transition.next().is_some()),
        )
        .map_err(|_| crate::runtime::focus::UiPortalFocusTransitionDenial::Routing)
    }

    pub(in crate::runtime) fn discard_portal_proposal(
        &mut self,
        proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
    ) -> Result<(), crate::runtime::focus::UiPortalFocusTransitionDenial> {
        let structural_revision = self
            .structural_revision
            .checked_add(1)
            .ok_or(crate::runtime::focus::UiPortalFocusTransitionDenial::Routing)?;
        drop(
            self.pending_portal
                .remove(&proposal)
                .ok_or(crate::runtime::focus::UiPortalFocusTransitionDenial::UnknownProposal)?,
        );
        self.structural_revision = structural_revision;
        Ok(())
    }

    pub(crate) fn requires_focused_submit(&self) -> bool {
        !self.portal_restorations.is_empty()
    }
}

fn initial_portal_target(
    ordered: &[crate::runtime::focus::UiFocusParticipant],
    current_index: &std::collections::BTreeMap<
        crate::runtime::focus::UiFocusParticipantIdentity,
        (crate::runtime::focus::UiFocusScopeIdentity, usize),
    >,
    portal_owner: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> Option<crate::runtime::focus::UiFocusParticipant> {
    ordered
        .iter()
        .copied()
        .find(|participant| {
            !current_index.contains_key(&participant.identity())
                && participant.identity().mounted_instance() != portal_owner
        })
        .or_else(|| {
            ordered
                .iter()
                .copied()
                .find(|participant| participant.identity().mounted_instance() == portal_owner)
        })
        .or_else(|| ordered.first().copied())
}

fn restoration_target(
    ordered: &[crate::runtime::focus::UiFocusParticipant],
    token: Option<crate::runtime::focus::UiFocusRestorationToken>,
) -> Option<crate::runtime::focus::UiFocusParticipant> {
    let token = token?;
    ordered
        .iter()
        .copied()
        .find(|participant| {
            participant.scope() == token.scope()
                && participant.identity() == token.participant()
                && participant.incarnation() == token.incarnation()
        })
        .or_else(|| {
            ordered
                .iter()
                .copied()
                .find(|participant| participant.scope() == token.scope())
        })
}

fn current_target(
    ordered: &[crate::runtime::focus::UiFocusParticipant],
    current: Option<crate::runtime::focus::UiSemanticKeyboardFocus>,
) -> Option<crate::runtime::focus::UiFocusParticipant> {
    let current = current?;
    ordered.iter().copied().find(|participant| {
        participant.scope() == current.scope()
            && participant.identity() == current.participant()
            && participant.incarnation() == current.incarnation()
    })
}
