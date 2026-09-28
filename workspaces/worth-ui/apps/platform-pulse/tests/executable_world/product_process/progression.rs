use crate::adjudication::{
    ExecutableFirstFrameEvidence, ExecutableLifecycleCleanupEvidence, VerticalThumbEvidence,
};
use crate::external_observation::PlatformPulseLifecycleStream;
use crate::failure_teardown::{NativeBoundFailureWorldResources, UnboundFailureWorldResources};
use crate::installation::IsolatedPulseInstallation;
use crate::native_platform::{CertifiedNativePlatform, CertifiedProcessBoundNativeClientArea};

use super::LivePlatformPulseProcess;

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
}

pub(crate) struct NativeBoundExecutableWorld {
    pub(super) installation: IsolatedPulseInstallation,
    pub(super) process: LivePlatformPulseProcess,
    pub(super) lifecycle: PlatformPulseLifecycleStream,
    pub(super) platform: CertifiedNativePlatform,
    pub(super) native_client: CertifiedProcessBoundNativeClientArea,
}

pub(crate) struct Published<Stage> {
    pub(super) world: NativeBoundExecutableWorld,
    pub(super) stage: Stage,
}

/// The source-signal first frame, adjudicated against its canonical color.
pub(crate) struct InitialBlue;

/// The dashboard's first frame, adjudicated by its resting Recent activity
/// thumb rather than the source-signal color.
pub(crate) struct DashboardAtRest {
    pub(super) evidence: ExecutableFirstFrameEvidence<VerticalThumbEvidence>,
}

pub(crate) struct Closed {
    pub(super) evidence: ExecutableLifecycleCleanupEvidence,
}

impl PulseExecutableWorld<Published<DashboardAtRest>> {
    pub(crate) fn evidence(&self) -> &ExecutableFirstFrameEvidence<VerticalThumbEvidence> {
        &self.state.stage.evidence
    }
}

impl PulseExecutableWorld<Closed> {
    pub(crate) fn evidence(&self) -> &ExecutableLifecycleCleanupEvidence {
        &self.state.evidence
    }
}

impl NativeBoundExecutableWorld {
    pub(super) fn into_failure_resources(self) -> NativeBoundFailureWorldResources {
        UnboundFailureWorldResources::new(self.installation, self.process, self.lifecycle)
            .bind_native(self.platform, self.native_client)
    }
}
