use crate::adjudication::{
    ExecutableFirstFrameEvidence, ExecutableLifecycleCleanupEvidence,
    ExecutableNativeInputReachabilityEvidence, ExecutablePredecessorPreservationEvidence,
    ExecutableReplacementEvidence, ExecutableVisualClearEvidence, ExecutableVisualOverlayEvidence,
    ExecutableVisualRetirementEvidence, ExecutableVisualSnapshotEvidence,
    ExecutableVisualTraceEvidence, VerticalThumbEvidence,
};
use crate::external_observation::PlatformPulseLifecycleStream;
use crate::failure_teardown::{NativeBoundFailureWorldResources, UnboundFailureWorldResources};
use crate::installation::IsolatedPulseInstallation;
use crate::native_platform::{WindowsNativePlatform, WindowsProcessBoundNativeClientArea};
use crate::source_delta::{
    AppliedPulseSourceDelta, CanonicalBlueRecoverySourceDelta, GreenPulseSourceDelta,
    MalformedPulseSourceDelta, QueryStatusV1, QueryStatusV2, RevisionSchemaSourceDelta,
    StatusSchemaRecoverySourceDelta,
};
use std::time::{Duration, Instant};

use super::LivePlatformPulseProcess;

mod final_recovery_evidence;

pub(crate) struct PulseExecutableWorld<State> {
    pub(super) state: State,
}

pub(crate) struct Installed {
    pub(super) installation: IsolatedPulseInstallation,
}

pub(crate) struct AwaitingFirstFrame {
    pub(super) installation: IsolatedPulseInstallation,
    pub(super) process: LivePlatformPulseProcess,
    pub(super) lifecycle: PlatformPulseLifecycleStream,
    pub(super) launch_started: Instant,
}

pub(crate) struct NativeBoundExecutableWorld {
    pub(super) installation: IsolatedPulseInstallation,
    pub(super) process: LivePlatformPulseProcess,
    pub(super) lifecycle: PlatformPulseLifecycleStream,
    pub(super) journey_started: Instant,
    pub(super) platform: WindowsNativePlatform,
    pub(super) native_client: WindowsProcessBoundNativeClientArea,
}

pub(crate) struct Published<Stage> {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) stage: Stage,
}

pub(crate) struct InitialBlue {
    pub(super) evidence: ExecutableFirstFrameEvidence,
    pub(super) launch_to_first_publication: Duration,
}

/// The dashboard's first frame, adjudicated by its resting Recent activity
/// thumb rather than the source-signal colour.
pub(crate) struct DashboardAtRest {
    pub(super) evidence: ExecutableFirstFrameEvidence<VerticalThumbEvidence>,
}

pub(crate) struct NativeInputReached<Stage> {
    pub(super) prior: Stage,
    pub(super) evidence: ExecutableNativeInputReachabilityEvidence,
}

impl NativeInputReached<InitialBlue> {
    pub(super) fn first_frame_evidence(&self) -> &ExecutableFirstFrameEvidence {
        &self.prior.evidence
    }
}

pub(crate) struct AwaitingQueryCurrent<Stage, Kind> {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) prior: Stage,
    pub(super) action: AppliedPulseSourceDelta<Kind>,
}

pub(crate) struct QueryCurrent<Stage, Kind> {
    pub(super) prior: Stage,
    pub(super) action: AppliedPulseSourceDelta<Kind>,
    pub(super) evidence: crate::adjudication::ExecutableQueryCurrentEvidence,
}

pub(crate) type FirstCurrent = QueryCurrent<NativeInputReached<InitialBlue>, QueryStatusV1>;
pub(crate) type SecondQueryCurrent = QueryCurrent<OverlayCleared<FirstCurrent>, QueryStatusV2>;

pub(crate) struct ComparisonBasisRefreshed<Stage> {
    pub(super) prior: Stage,
    pub(super) retirement: ExecutableVisualRetirementEvidence,
    pub(super) snapshot: ExecutableVisualSnapshotEvidence,
}

pub(crate) type SecondCurrent = ComparisonBasisRefreshed<SecondQueryCurrent>;
pub(crate) type PortalReady = SecondCurrent;

pub(crate) struct SnapshotCaptured<Stage> {
    pub(super) prior: Stage,
    pub(super) evidence: ExecutableVisualSnapshotEvidence,
}

pub(crate) struct IdentityTraced<Stage> {
    pub(super) snapshot: SnapshotCaptured<Stage>,
    pub(super) evidence: ExecutableVisualTraceEvidence,
}

pub(crate) struct OverlayPublished<Stage> {
    pub(super) trace: IdentityTraced<Stage>,
    pub(super) evidence: ExecutableVisualOverlayEvidence,
}

pub(crate) struct OverlayCleared<Stage> {
    pub(super) overlay: OverlayPublished<Stage>,
    pub(super) evidence: ExecutableVisualClearEvidence,
}

impl OverlayCleared<FirstCurrent> {
    pub(super) fn initial(&self) -> &FirstCurrent {
        &self.overlay.trace.snapshot.prior
    }

    pub(super) fn snapshot_evidence(&self) -> &ExecutableVisualSnapshotEvidence {
        &self.overlay.trace.snapshot.evidence
    }
}

pub(crate) struct AwaitingReplacement {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) initial: SecondCurrent,
    pub(super) action: AppliedPulseSourceDelta<GreenPulseSourceDelta>,
}

pub(crate) struct GreenSuccessor {
    pub(super) initial: SecondCurrent,
    pub(super) evidence: ExecutableReplacementEvidence<GreenPulseSourceDelta>,
    pub(super) snapshot: ExecutableVisualSnapshotEvidence,
    pub(super) retirement: ExecutableVisualRetirementEvidence,
}

pub(crate) struct AwaitingPreservation {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) green: GreenSuccessor,
    pub(super) action: AppliedPulseSourceDelta<MalformedPulseSourceDelta>,
}

pub(crate) struct PreservedPredecessor {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) green: GreenSuccessor,
    pub(super) evidence: ExecutablePredecessorPreservationEvidence,
}

pub(crate) struct AwaitingRecovery {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) preserved: PreservedPredecessorEvidence,
    pub(super) action: AppliedPulseSourceDelta<CanonicalBlueRecoverySourceDelta>,
}

pub(crate) struct PreservedPredecessorEvidence {
    pub(super) green: GreenSuccessor,
    pub(super) evidence: ExecutablePredecessorPreservationEvidence,
}

pub(crate) struct RecoveredBlue {
    pub(super) preserved: PreservedPredecessorEvidence,
    pub(super) evidence: ExecutableReplacementEvidence<CanonicalBlueRecoverySourceDelta>,
    pub(super) rebase_snapshot: ExecutableVisualSnapshotEvidence,
}

pub(crate) struct AwaitingSchemaStop {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) recovered: RecoveredBlue,
    pub(super) action: AppliedPulseSourceDelta<RevisionSchemaSourceDelta>,
}

pub(crate) struct SchemaStopped {
    pub(super) recovered: RecoveredBlue,
    pub(super) evidence:
        crate::adjudication::ExecutableSchemaTransitionEvidence<RevisionSchemaSourceDelta>,
    pub(super) retirement: ExecutableVisualRetirementEvidence,
    pub(super) snapshot: ExecutableVisualSnapshotEvidence,
}

pub(crate) struct AwaitingStatusRecovery {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) stopped: SchemaStopped,
    pub(super) action: AppliedPulseSourceDelta<StatusSchemaRecoverySourceDelta>,
}

pub(crate) struct FinalRecovered {
    pub(super) stopped: SchemaStopped,
    pub(super) evidence:
        crate::adjudication::ExecutableSchemaTransitionEvidence<StatusSchemaRecoverySourceDelta>,
    pub(super) rebase_snapshot: ExecutableVisualSnapshotEvidence,
}

pub(crate) struct Closed {
    pub(super) evidence: ExecutableLifecycleCleanupEvidence,
}

impl PulseExecutableWorld<Published<InitialBlue>> {
    pub(crate) fn evidence(&self) -> &ExecutableFirstFrameEvidence {
        &self.state.stage.evidence
    }

    pub(crate) fn launch_to_first_publication(&self) -> Duration {
        self.state.stage.launch_to_first_publication
    }
}

impl PulseExecutableWorld<Published<DashboardAtRest>> {
    pub(crate) fn evidence(&self) -> &ExecutableFirstFrameEvidence<VerticalThumbEvidence> {
        &self.state.stage.evidence
    }
}

impl PulseExecutableWorld<Published<NativeInputReached<InitialBlue>>> {
    pub(crate) fn evidence(&self) -> &ExecutableNativeInputReachabilityEvidence {
        &self.state.stage.evidence
    }
}

impl<Stage, Kind> PulseExecutableWorld<Published<QueryCurrent<Stage, Kind>>> {
    pub(crate) fn query_evidence(&self) -> &crate::adjudication::ExecutableQueryCurrentEvidence {
        &self.state.stage.evidence
    }
}

impl PulseExecutableWorld<Published<SecondCurrent>> {
    pub(crate) fn query_evidence(&self) -> &crate::adjudication::ExecutableQueryCurrentEvidence {
        &self.state.stage.prior.evidence
    }

    pub(crate) fn refresh_retirement_evidence(&self) -> ExecutableVisualRetirementEvidence {
        self.state.stage.retirement
    }

    pub(crate) fn refresh_snapshot_evidence(&self) -> &ExecutableVisualSnapshotEvidence {
        &self.state.stage.snapshot
    }
}

impl ComparisonBasisRefreshed<SecondQueryCurrent> {
    pub(super) fn visual(&self) -> &OverlayCleared<FirstCurrent> {
        &self.prior.prior
    }

    pub(super) fn snapshot_evidence(&self) -> &ExecutableVisualSnapshotEvidence {
        &self.snapshot
    }

    pub(super) fn canonical_source_digest(&self) -> u64 {
        self.visual()
            .initial()
            .prior
            .first_frame_evidence()
            .first_frame()
            .source()
            .final_package_digest()
    }
}

impl PulseExecutableWorld<Published<SnapshotCaptured<FirstCurrent>>> {
    pub(crate) fn evidence(&self) -> &ExecutableVisualSnapshotEvidence {
        &self.state.stage.evidence
    }
}

impl PulseExecutableWorld<Published<IdentityTraced<FirstCurrent>>> {
    pub(crate) fn evidence(&self) -> &ExecutableVisualTraceEvidence {
        &self.state.stage.evidence
    }
}

impl PulseExecutableWorld<Published<OverlayPublished<FirstCurrent>>> {
    pub(crate) fn evidence(&self) -> &ExecutableVisualOverlayEvidence {
        &self.state.stage.evidence
    }
}

impl PulseExecutableWorld<Published<OverlayCleared<FirstCurrent>>> {
    pub(crate) fn evidence(&self) -> &ExecutableVisualClearEvidence {
        &self.state.stage.evidence
    }
}

impl PulseExecutableWorld<Published<GreenSuccessor>> {
    pub(crate) fn evidence(&self) -> &ExecutableReplacementEvidence<GreenPulseSourceDelta> {
        &self.state.stage.evidence
    }

    pub(crate) fn retirement_evidence(&self) -> ExecutableVisualRetirementEvidence {
        self.state.stage.retirement
    }

    pub(crate) fn snapshot_evidence(&self) -> &ExecutableVisualSnapshotEvidence {
        &self.state.stage.snapshot
    }
}

impl PulseExecutableWorld<PreservedPredecessor> {
    pub(crate) fn evidence(&self) -> &ExecutablePredecessorPreservationEvidence {
        &self.state.evidence
    }
}

impl PulseExecutableWorld<Published<RecoveredBlue>> {
    pub(crate) fn evidence(
        &self,
    ) -> &ExecutableReplacementEvidence<CanonicalBlueRecoverySourceDelta> {
        &self.state.stage.evidence
    }

    pub(crate) fn preservation_evidence(&self) -> &ExecutablePredecessorPreservationEvidence {
        &self.state.stage.preserved.evidence
    }

    pub(crate) fn rebase_snapshot_evidence(&self) -> &ExecutableVisualSnapshotEvidence {
        &self.state.stage.rebase_snapshot
    }

    pub(crate) fn query_basis(
        &self,
    ) -> &worth_ui_platform_pulse::observation_contract::PlatformPulseQueryProjectionEvidence {
        self.state
            .stage
            .preserved
            .green
            .initial
            .prior
            .evidence
            .projection()
    }
}

impl PulseExecutableWorld<Published<SchemaStopped>> {
    pub(crate) fn evidence(
        &self,
    ) -> &crate::adjudication::ExecutableSchemaTransitionEvidence<RevisionSchemaSourceDelta> {
        &self.state.stage.evidence
    }

    pub(crate) fn retirement_evidence(&self) -> ExecutableVisualRetirementEvidence {
        self.state.stage.retirement
    }

    pub(crate) fn recovered_snapshot_evidence(&self) -> &ExecutableVisualSnapshotEvidence {
        &self.state.stage.recovered.rebase_snapshot
    }
}

impl PulseExecutableWorld<Closed> {
    pub(crate) fn evidence(&self) -> &ExecutableLifecycleCleanupEvidence {
        &self.state.evidence
    }
}

impl<Stage> PulseExecutableWorld<Published<Stage>> {
    pub(crate) fn native_journey_started(&self) -> Instant {
        self.state.world.journey_started
    }
}

impl NativeBoundExecutableWorld {
    pub(super) fn into_failure_resources(self) -> NativeBoundFailureWorldResources {
        UnboundFailureWorldResources::new(self.installation, self.process, self.lifecycle)
            .bind_native(self.platform, self.native_client)
    }
}
