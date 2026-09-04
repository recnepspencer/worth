use crate::runtime::WorthUiRuntimeDiagnosticPolicy;
use crate::runtime::WorthUiRuntimeFrameEpoch;
use crate::source::WorthUiArtifact;
use std::rc::Rc;

/// Launch request for creating an active runtime instance from canonical candidate truth.
#[derive(Debug)]
pub struct WorthUiRuntimeLaunch {
    pub(crate) artifact: Rc<WorthUiArtifact>,
    pub(crate) frame_epoch: WorthUiRuntimeFrameEpoch,
    pub(crate) diagnostic_policy: WorthUiRuntimeDiagnosticPolicy,
    pub(crate) candidate_snapshot_digest: Option<u64>,
    pub(crate) candidate_artifact_digest: Option<crate::source::WorthUiArtifactDigest>,
}

/// Move-only prepared authority required to turn one admitted launch request
/// into a runtime. Constituent proofs never cross the public facade
/// independently.
pub(crate) struct WorthUiRuntimeLaunchAuthority {
    pub(crate) lowering_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationLoweringAuthority,
    pub(crate) initial_allocation_commit:
        crate::runtime::planning::allocation_planning::WorthUiInitialAllocationCommit,
    pub(crate) snapshot_digest: crate::capability::CapabilitySnapshotDigest,
    pub(crate) retained_allocation_planning_evidence:
        Rc<crate::runtime::planning::allocation_planning::WorthUiRetainedAllocationPlanningEvidenceRegistry>,
    pub(crate) query_binding: worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    pub(crate) host_plan_binding: crate::facade::WorthUiHostPlanBinding,
    pub(crate) change_profile: crate::runtime::rebind::UiChangeProfile,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorthUiRuntimeLaunchDenial {
    HostSessionIdentityExhausted,
    HostProtocol(worth_ui_host_contract::UiHostProtocolDenial),
    HostMountedPresentationLease,
    HostObservationSession(worth_ui_host_contract::UiHostObservationSessionRegistrationDenial),
    HostSessionReleaseIndeterminate {
        cause: Box<WorthUiRuntimeLaunchDenial>,
        recovery: Box<crate::facade::WorthUiHostSessionReleaseRecovery>,
    },
    HostSessionReleaseMismatch {
        cause: Box<WorthUiRuntimeLaunchDenial>,
        released_surface_count: usize,
    },
    MountedIdentityExhausted,
    AppearanceThemeAdmission(crate::runtime::appearance::UiThemeCapabilityReceiptDenial),
    AppearanceOwnerUnavailable(worth_ui_dsl::UiAppearanceStateAxis),
    InitialAllocationGraphAuthorityMismatch,
    InitialAllocationObligationsUnsettled {
        node_count: usize,
    },
    CandidateGraphAuthorityMismatch,
    CandidateArtifactAuthorityMismatch,
    ForeignAllocationProjection,
    MissingQueryPosture,
    UnexpectedQueryPosture,
    QueryDefinitionNotInstalled,
    ForeignQueryInstalledAuthority,
    RegionalDeltaDuplicateCandidateRegion,
    PlanInput(crate::runtime::WorthUiPlanLoweringDenial),
    HandleAllocation(crate::runtime::WorthUiRuntimeHandleAllocationDenial),
    TopologyAssembly(crate::runtime::WorthUiPlanTopologyDenial),
    ExecutionPlanAuthorityMismatch,
    OrdinaryPlan(crate::runtime::WorthUiOrdinaryLanePlanDenial),
    VirtualizedPlan(crate::runtime::WorthUiVirtualizedDataPlanDenial),
    CanvasSpatialPlan(crate::runtime::WorthUiCanvasSpatialPlanDenial),
    RealtimeOverlayPlan(crate::runtime::WorthUiHudPlanDenial),
    StalePendingActivation {
        pending_epoch: WorthUiRuntimeFrameEpoch,
        active_epoch: WorthUiRuntimeFrameEpoch,
    },
    CandidateSnapshotMismatch {
        candidate_snapshot_digest: u64,
        app_snapshot_digest: u64,
    },
}

impl WorthUiRuntimeLaunchDenial {
    pub(crate) fn retry_host_session_cleanup(
        self,
    ) -> Result<WorthUiRuntimeLaunchDenial, WorthUiRuntimeLaunchDenial> {
        if matches!(&self, Self::HostSessionReleaseMismatch { .. }) {
            return Err(self);
        }
        let Self::HostSessionReleaseIndeterminate { cause, recovery } = self else {
            return Ok(self);
        };
        match (*recovery).retry() {
            Ok(_) => Ok(*cause),
            Err(recovery) => Err(Self::HostSessionReleaseIndeterminate {
                cause,
                recovery: Box::new(recovery),
            }),
        }
    }
}

impl WorthUiRuntimeLaunch {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn from_canonical_artifact(artifact: WorthUiArtifact) -> Self {
        Self {
            artifact: Rc::new(artifact),
            frame_epoch: WorthUiRuntimeFrameEpoch::initial(),
            diagnostic_policy: WorthUiRuntimeDiagnosticPolicy::minimal(),
            candidate_snapshot_digest: None,
            candidate_artifact_digest: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_candidate(candidate: crate::runtime::WorthUiReplacementCandidate) -> Self {
        let candidate_snapshot_digest = candidate.lowering_basis().snapshot_digest();
        let candidate_artifact_digest = candidate.basis().artifact_digest();
        Self {
            artifact: candidate.artifact_bundle().artifact_authority(),
            frame_epoch: WorthUiRuntimeFrameEpoch::initial(),
            diagnostic_policy: WorthUiRuntimeDiagnosticPolicy::minimal(),
            candidate_snapshot_digest: Some(candidate_snapshot_digest),
            candidate_artifact_digest: Some(candidate_artifact_digest),
        }
    }

    pub fn with_diagnostics(mut self, diagnostic_policy: WorthUiRuntimeDiagnosticPolicy) -> Self {
        self.diagnostic_policy = diagnostic_policy;
        self
    }
}
