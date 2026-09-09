use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionStagingDenial {
    CapacityExceeded,
    TrackIdentityExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiMotionCommitDenial {
    PreparedFrameMismatch,
    PublicationProposalMismatch,
    PublicationRejected,
    Census,
}

/// Canonical semantic motion-track owner. Presentation sampling receives only
/// committed track copies and cannot mutate this table.
pub(crate) struct UiMotionRuntimeState {
    persistence: crate::runtime::UiServiceStatePersistencePosture,
    policy: crate::declaration::UiMotionPolicy,
    next_track_identity: u64,
    pub(super) tracks: BTreeMap<super::UiMotionTargetIdentity, super::UiCommittedMotionTrack>,
    pub(super) overlay_rows: BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        super::overlay_export::UiMotionOverlayRows,
    >,
    exit_retentions: BTreeMap<super::UiMotionTrackIdentity, super::UiMotionExitRetentionReceipt>,
    census: super::UiMotionResourceCensus,
    publication_sequence: u64,
    last_fact: Option<super::UiMotionProducedFact>,
    last_interruption: Option<super::UiMotionProducedFact>,
}

impl UiMotionRuntimeState {
    pub(crate) const fn new(persistence: crate::runtime::UiServiceStatePersistencePosture) -> Self {
        Self::new_with_policy(
            persistence,
            crate::declaration::UiMotionPolicy::system_respecting(),
        )
    }

    pub(crate) const fn new_with_policy(
        persistence: crate::runtime::UiServiceStatePersistencePosture,
        policy: crate::declaration::UiMotionPolicy,
    ) -> Self {
        Self {
            persistence,
            policy,
            next_track_identity: 1,
            tracks: BTreeMap::new(),
            overlay_rows: BTreeMap::new(),
            exit_retentions: BTreeMap::new(),
            census: super::UiMotionResourceCensus::zero(),
            publication_sequence: 0,
            last_fact: None,
            last_interruption: None,
        }
    }

    pub(crate) fn apply_policy(&mut self, policy: crate::declaration::UiMotionPolicy) {
        self.policy = policy;
    }

    pub(in crate::runtime) fn stage(
        &mut self,
        proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
        request: super::UiMotionTransitionRequest,
    ) -> Result<super::UiStagedMotionServiceProposal, UiMotionStagingDenial> {
        let request = request.with_policy(self.policy);
        self.census
            .stage()
            .map_err(|_| UiMotionStagingDenial::CapacityExceeded)?;
        let Some(identity) = super::UiMotionTrackIdentity::allocate(self.next_track_identity)
        else {
            self.rollback_staging();
            return Err(UiMotionStagingDenial::TrackIdentityExhausted);
        };
        self.next_track_identity = match self.next_track_identity.checked_add(1) {
            Some(next) => next,
            None => {
                self.rollback_staging();
                return Err(UiMotionStagingDenial::TrackIdentityExhausted);
            }
        };
        let scope = crate::runtime::session::service_proposal::
            UiServiceProposalOccupancyScopeIdentity::for_mounted_owner(
                request.successor().target().mounted_instance(),
            );
        Ok(super::UiStagedMotionServiceProposal {
            identity,
            proposal,
            scope,
            request,
            fact: crate::runtime::session::service_proposal::UiServiceProducedFactReference::for_motion_proposal(
                proposal,
                scope,
            ),
        })
    }

    pub(in crate::runtime) fn derive(
        &self,
        staged: super::UiStagedMotionServiceProposal,
        prepared_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) -> super::UiDerivedMotionServiceProposal {
        super::UiDerivedMotionServiceProposal {
            staged: Box::new(staged),
            prepared_frame,
        }
    }

    pub(in crate::runtime) fn discard_staged(
        &mut self,
        _staged: super::UiStagedMotionServiceProposal,
    ) {
        self.rollback_staging();
    }

    pub(in crate::runtime) fn discard_derived(
        &mut self,
        _derived: super::UiDerivedMotionServiceProposal,
    ) {
        self.rollback_staging();
    }

    pub(in crate::runtime) fn commit_published(
        &mut self,
        mut derived: super::UiDerivedMotionServiceProposal,
        publication: crate::runtime::session::service_proposal::UiServiceProposalPublicationReceipt,
        mounted_frame: worth_ui_host_contract::UiMountedFrameIdentity,
        mounted_presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        super::UiMotionCommitReceipt,
        (super::UiDerivedMotionServiceProposal, UiMotionCommitDenial),
    > {
        if derived.prepared_frame != mounted_frame {
            return Err((derived, UiMotionCommitDenial::PreparedFrameMismatch));
        }
        if derived.staged.proposal != publication.proposal() {
            return Err((derived, UiMotionCommitDenial::PublicationProposalMismatch));
        }
        if publication.disposition()
            != crate::runtime::session::service_proposal::UiServiceProposalPublicationDisposition::Accepted
        {
            return Err((derived, UiMotionCommitDenial::PublicationRejected));
        }
        derived.staged.request = match derived
            .staged
            .request
            .bind_published_successor(mounted_presentation)
        {
            Ok(request) => request,
            Err(_) => return Err((derived, UiMotionCommitDenial::PreparedFrameMismatch)),
        };
        let target = derived.staged.request.successor().target();
        let retarget = self
            .tracks
            .get(&target)
            .map(|_| super::retarget::resolve(derived.staged.request.declaration().interruption()));
        let displaced_exit_retention = self
            .exit_retentions
            .values()
            .find(|retention| retention.target() == target)
            .copied();
        let retains_exit =
            derived.staged.request.declaration().fill() == super::UiMotionFillPolicy::ExitRetention;
        let reserves_exit_retention = retains_exit && displaced_exit_retention.is_none();
        if reserves_exit_retention && self.census.reserve_exit_retention().is_err() {
            return Err((derived, UiMotionCommitDenial::Census));
        }
        if self.census.commit_staged(retarget.is_some()).is_err() {
            if reserves_exit_retention {
                self.census
                    .release_exit_retention()
                    .expect("same commit reserved the exit-retention census");
            }
            return Err((derived, UiMotionCommitDenial::Census));
        }
        let track = super::UiCommittedMotionTrack::new(&derived, retarget);
        let exit_retention = retains_exit.then(|| super::UiMotionExitRetentionReceipt::new(track));
        if let Some(displaced) = displaced_exit_retention {
            assert_eq!(
                self.exit_retentions.remove(&displaced.track()),
                Some(displaced),
                "Motion interruption displaces its exact retained exit"
            );
            if !retains_exit {
                self.census
                    .release_exit_retention()
                    .expect("displaced Motion exit owns one census entry");
            }
        }
        if let Some(exit_retention) = exit_retention {
            self.exit_retentions
                .insert(track.identity(), exit_retention);
        }
        let kind = retarget.map_or(
            super::UiMotionProducedFactKind::Started,
            super::UiMotionProducedFactKind::Retargeted,
        );
        let fact = self.publish(track.identity(), track.request(), kind);
        self.tracks.insert(target, track);
        Ok(super::UiMotionCommitReceipt::new(
            track,
            fact,
            exit_retention,
            displaced_exit_retention,
        ))
    }

    pub(crate) fn terminalize(
        &mut self,
        track: super::UiMotionTrackIdentity,
        cause: super::UiMotionTerminalCause,
    ) -> Option<super::UiMotionTerminalReceipt> {
        let target = self
            .tracks
            .iter()
            .find_map(|(target, candidate)| (candidate.identity() == track).then_some(*target))?;
        self.terminalize_target(target, cause)
    }

    pub(super) fn terminalize_target(
        &mut self,
        target: super::UiMotionTargetIdentity,
        cause: super::UiMotionTerminalCause,
    ) -> Option<super::UiMotionTerminalReceipt> {
        let track = self.tracks.remove(&target)?;
        self.census
            .terminal()
            .expect("terminal Motion track retains one active census entry");
        let fact = self.publish(
            track.identity(),
            track.request(),
            super::UiMotionProducedFactKind::Terminal(cause),
        );
        let exit_retention = self.exit_retentions.get(&track.identity()).copied();
        Some(super::UiMotionTerminalReceipt::new(
            track,
            cause,
            fact,
            exit_retention,
        ))
    }

    pub(crate) fn release_exit_retention(
        &mut self,
        retention: super::UiMotionExitRetentionReceipt,
    ) -> bool {
        if self.exit_retentions.get(&retention.track()) != Some(&retention) {
            return false;
        }
        self.exit_retentions.remove(&retention.track());
        self.census
            .release_exit_retention()
            .expect("retained Motion exit owns one census entry");
        true
    }

    fn publish(
        &mut self,
        track: super::UiMotionTrackIdentity,
        request: super::UiMotionTransitionRequest,
        kind: super::UiMotionProducedFactKind,
    ) -> super::UiMotionProducedFact {
        self.refresh_overlay_row(request, kind);
        self.publication_sequence = self
            .publication_sequence
            .checked_add(1)
            .expect("bounded Motion fact sequence exhausted");
        let fact =
            super::UiMotionProducedFact::new(self.publication_sequence, track, request, kind);
        if matches!(kind, super::UiMotionProducedFactKind::Retargeted(_)) {
            self.last_interruption = Some(fact);
        }
        self.last_fact = Some(fact);
        fact
    }

    fn rollback_staging(&mut self) {
        self.census
            .discard_staged()
            .expect("discarded Motion proposal retains one census entry");
    }

    pub(crate) const fn census(&self) -> super::UiMotionResourceCensus {
        self.census
    }

    pub(crate) const fn publication_count(&self) -> u64 {
        self.publication_sequence
    }

    #[cfg(test)]
    pub(crate) fn commit_declared_transition_for_test(
        &mut self,
        identity: u64,
        request: super::UiMotionTransitionRequest,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> super::UiMotionCommitReceipt {
        let proposal =
            crate::runtime::session::service_proposal::UiServiceProposalIdentity::for_test(
                identity,
            );
        let staged = self.stage(proposal, request).unwrap();
        let derived = self.derive(staged, frame);
        let publication = crate::runtime::session::service_proposal::
            UiServiceProposalPublicationReceipt::recorded_foreign_fixture(
                proposal,
                identity,
                crate::runtime::session::service_proposal::
                    UiServiceProposalPublicationDisposition::Accepted,
            );
        self.commit_published(derived, publication, frame, presentation)
            .unwrap_or_else(|(_, denial)| panic!("declared test transition commits: {denial:?}"))
    }

    #[cfg(test)]
    pub(crate) const fn last_fact(&self) -> Option<super::UiMotionProducedFact> {
        self.last_fact
    }

    pub(crate) const fn last_interruption(&self) -> Option<super::UiMotionProducedFact> {
        self.last_interruption
    }

    pub(crate) fn shutdown(&mut self) -> super::UiMotionShutdownReport {
        debug_assert_eq!(
            self.persistence,
            crate::runtime::UiServiceStatePersistencePosture::Ephemeral
        );
        let census_before_shutdown = self.census;
        let targets: Vec<_> = self.tracks.keys().copied().collect();
        for target in targets {
            self.terminalize_target(target, super::UiMotionTerminalCause::ApplicationShutdown)
                .expect("shutdown target was retained by the canonical Motion table");
        }
        let report = super::UiMotionShutdownReport::from_census(census_before_shutdown);
        self.exit_retentions.clear();
        self.last_interruption = None;
        self.census = report.final_census();
        report
    }
}
