use core::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiMotionTrackIdentity(NonZeroU64);

#[must_use = "the Motion owner must derive, commit, or discard every staged proposal"]
pub(in crate::runtime) struct UiStagedMotionServiceProposal {
    pub(super) identity: UiMotionTrackIdentity,
    pub(super) proposal: crate::runtime::session::service_proposal::UiServiceProposalIdentity,
    pub(super) scope:
        crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
    pub(super) request: super::UiMotionTransitionRequest,
    pub(super) fact: crate::runtime::session::service_proposal::UiServiceProducedFactReference,
}

#[must_use = "a derived Motion proposal must settle with existing mounted publication"]
pub(in crate::runtime) struct UiDerivedMotionServiceProposal {
    pub(super) staged: Box<UiStagedMotionServiceProposal>,
    pub(super) prepared_frame: worth_ui_host_contract::UiMountedFrameIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiCommittedMotionTrack {
    identity: UiMotionTrackIdentity,
    request: super::UiMotionTransitionRequest,
    retarget: Option<super::UiMotionRetargetDisposition>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionCommitReceipt {
    track: UiCommittedMotionTrack,
    fact: super::UiMotionProducedFact,
    exit_retention: Option<UiMotionExitRetentionReceipt>,
    displaced_exit_retention: Option<UiMotionExitRetentionReceipt>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionExitRetentionReceipt {
    track: UiMotionTrackIdentity,
    target: super::UiMotionTargetIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMotionTerminalCause {
    Completed,
    SnappedToTarget,
    ReboundAway,
    ApplicationShutdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMotionTerminalReceipt {
    track: UiCommittedMotionTrack,
    cause: UiMotionTerminalCause,
    fact: super::UiMotionProducedFact,
    exit_retention: Option<UiMotionExitRetentionReceipt>,
}

impl UiMotionTrackIdentity {
    pub(super) fn allocate(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    #[cfg(test)]
    pub(crate) fn for_test(value: u64) -> Self {
        Self(NonZeroU64::new(value).expect("test motion track identity must be non-zero"))
    }

    pub(crate) const fn diagnostic_value(self) -> u64 {
        self.0.get()
    }
}

impl UiStagedMotionServiceProposal {
    pub(in crate::runtime) const fn family_proposal(
        request: &super::UiMotionTransitionRequest,
    ) -> crate::runtime::session::service_proposal::UiServiceFamilyProposal {
        let scope = crate::runtime::session::service_proposal::
            UiServiceProposalOccupancyScopeIdentity::for_mounted_owner(
                request.successor().target().mounted_instance(),
            );
        crate::runtime::session::service_proposal::UiServiceFamilyProposal::motion(scope)
    }

    pub(in crate::runtime) const fn scope(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity {
        self.scope
    }

    pub(in crate::runtime) fn family_stage_receipt(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalStageReceipt {
        crate::runtime::session::service_proposal::UiServiceProposalStageReceipt::from_family_owner(
            self.proposal,
            crate::capability::UiRuntimeServiceFamily::Motion,
            self.scope,
            vec![self.fact],
            Vec::new(),
        )
    }
}

impl UiDerivedMotionServiceProposal {
    pub(in crate::runtime) fn overlay_row(&self) -> super::UiMotionOverlayOwnerRow {
        super::UiMotionOverlayOwnerRow::from_request(self.staged.request)
    }

    pub(in crate::runtime) fn derivation_receipt(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalStageReceipt {
        crate::runtime::session::service_proposal::UiServiceProposalStageReceipt::motion_derivation(
            self.staged.proposal,
        )
    }

    pub(in crate::runtime) const fn scope(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity {
        self.staged.scope
    }
}

impl UiCommittedMotionTrack {
    pub(super) const fn new(
        derived: &UiDerivedMotionServiceProposal,
        retarget: Option<super::UiMotionRetargetDisposition>,
    ) -> Self {
        Self {
            identity: derived.staged.identity,
            request: derived.staged.request,
            retarget,
        }
    }

    pub(crate) const fn identity(self) -> UiMotionTrackIdentity {
        self.identity
    }

    pub(in crate::runtime) const fn request(self) -> super::UiMotionTransitionRequest {
        self.request
    }

    pub(crate) const fn retarget(self) -> Option<super::UiMotionRetargetDisposition> {
        self.retarget
    }

    pub(crate) const fn target(self) -> super::UiMotionTargetIdentity {
        self.request.successor().target()
    }

    pub(crate) const fn predecessor_geometry(self) -> Option<super::UiMotionSemanticGeometry> {
        self.request.predecessor().geometry()
    }

    pub(crate) const fn predecessor_visible(self) -> bool {
        self.request.predecessor().visible()
    }

    pub(crate) const fn successor_geometry(self) -> Option<super::UiMotionSemanticGeometry> {
        self.request.successor().geometry()
    }

    pub(crate) const fn successor_visible(self) -> bool {
        self.request.successor().visible()
    }

    pub(crate) const fn successor_revision(self) -> u64 {
        self.request.successor().owner_revision()
    }

    pub(crate) const fn successor_presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.request.successor().presentation()
    }

    pub(crate) const fn declaration(self) -> super::UiMotionDeclaration {
        self.request.declaration()
    }

    pub(crate) fn rebind_published_presentation(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        self.request = self.request.rebind_published_successor(presentation);
        self
    }

    #[cfg(test)]
    pub(crate) fn for_sampling_test(
        identity: u64,
        request: super::UiMotionTransitionRequest,
        retarget: Option<super::UiMotionRetargetDisposition>,
    ) -> Self {
        Self {
            identity: UiMotionTrackIdentity::allocate(identity).expect("non-zero test identity"),
            request,
            retarget,
        }
    }
}

impl UiMotionCommitReceipt {
    pub(super) const fn new(
        track: UiCommittedMotionTrack,
        fact: super::UiMotionProducedFact,
        exit_retention: Option<UiMotionExitRetentionReceipt>,
        displaced_exit_retention: Option<UiMotionExitRetentionReceipt>,
    ) -> Self {
        Self {
            track,
            fact,
            exit_retention,
            displaced_exit_retention,
        }
    }

    pub(crate) const fn track(self) -> UiCommittedMotionTrack {
        self.track
    }

    #[cfg(test)]
    pub(crate) const fn fact(self) -> super::UiMotionProducedFact {
        self.fact
    }

    pub(crate) const fn exit_retention(self) -> Option<UiMotionExitRetentionReceipt> {
        self.exit_retention
    }

    pub(crate) const fn displaced_exit_retention(self) -> Option<UiMotionExitRetentionReceipt> {
        self.displaced_exit_retention
    }

    #[cfg(test)]
    pub(crate) fn for_sampling_test(track: UiCommittedMotionTrack) -> Self {
        Self {
            track,
            fact: super::UiMotionProducedFact::new(
                1,
                track.identity(),
                track.request(),
                super::UiMotionProducedFactKind::Started,
            ),
            exit_retention: None,
            displaced_exit_retention: None,
        }
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn for_sampling_test_transition(
        identity: u64,
        target: super::UiMotionTargetIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        predecessor_geometry: Option<[f32; 4]>,
        predecessor_visible: bool,
        successor_geometry: Option<[f32; 4]>,
        successor_visible: bool,
        declaration: super::UiMotionDeclaration,
        retarget: Option<super::UiMotionRetargetDisposition>,
    ) -> Self {
        let geometry = |components: Option<[f32; 4]>| {
            components.map(|components| {
                super::UiMotionSemanticGeometry::from_committed_components(
                    components,
                    worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
                )
                .expect("sampling test geometry is valid committed semantic geometry")
            })
        };
        let request = super::UiMotionTransitionRequest::from_family_transition(
            target,
            identity,
            identity + 1,
            presentation,
            geometry(predecessor_geometry),
            predecessor_visible,
            presentation,
            geometry(successor_geometry),
            successor_visible,
            declaration,
        )
        .expect("sampling test transition has an advancing revision and stable binding");
        Self::for_sampling_test(UiCommittedMotionTrack::for_sampling_test(
            identity, request, retarget,
        ))
    }
}

impl UiMotionTerminalReceipt {
    pub(super) const fn new(
        track: UiCommittedMotionTrack,
        cause: UiMotionTerminalCause,
        fact: super::UiMotionProducedFact,
        exit_retention: Option<UiMotionExitRetentionReceipt>,
    ) -> Self {
        Self {
            track,
            cause,
            fact,
            exit_retention,
        }
    }

    pub(crate) const fn track(self) -> UiMotionTrackIdentity {
        self.track.identity()
    }

    #[cfg(test)]
    pub(crate) const fn cause(self) -> UiMotionTerminalCause {
        self.cause
    }

    #[cfg(test)]
    pub(crate) const fn fact(self) -> super::UiMotionProducedFact {
        self.fact
    }

    pub(crate) const fn exit_retention(self) -> Option<UiMotionExitRetentionReceipt> {
        self.exit_retention
    }
}

impl UiMotionExitRetentionReceipt {
    pub(super) const fn new(track: UiCommittedMotionTrack) -> Self {
        Self {
            track: track.identity(),
            target: track.target(),
        }
    }

    pub(crate) const fn track(self) -> UiMotionTrackIdentity {
        self.track
    }

    pub(crate) const fn target(self) -> super::UiMotionTargetIdentity {
        self.target
    }
}
