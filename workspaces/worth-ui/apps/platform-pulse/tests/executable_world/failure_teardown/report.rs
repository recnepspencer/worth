use std::fmt;

use crate::adjudication::{
    ExecutableFirstFrameFailure, ExecutableLifecycleCleanupFailure,
    ExecutableNativeInputReachabilityFailure, ExecutablePredecessorPreservationFailure,
    ExecutableReplacementFailure, ExecutableSchemaTransitionFailure,
    ExecutableVisualIdentityFailure,
};
use crate::external_observation::{
    PlatformPulseLifecycleStreamFailure, PlatformPulseLifecycleTeardownEvidence,
    PlatformPulseLifecycleTeardownFailure, StableProcessLivenessFailure,
};
use crate::installation::{
    PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure, PulseInstallationFailure,
};
use crate::native_platform::NativePlatformFailure;
use crate::product_process::{
    EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure,
    PlatformPulseNativeCloseEvidenceFailure, PlatformPulseProcessExitFailure,
    PlatformPulseProcessLaunchFailure, PlatformPulseQuiescenceFailure,
    WatchedPulseObservationFailure,
};
use crate::source_delta::PulseSourceActionFailure;

use super::retained_artifact::{
    FailureArtifactDiscardEvidence, FailureArtifactFailure, FailureArtifactInputs,
    RetainedFailureArtifact,
};

#[derive(Debug)]
pub(crate) enum PulseExecutableWorldFailure {
    Installation(PulseInstallationFailure),
    InstallationCleanup(PulseInstallationCleanupFailure),
    Launch(PlatformPulseProcessLaunchFailure),
    Lifecycle(PlatformPulseLifecycleStreamFailure),
    Liveness(StableProcessLivenessFailure),
    Native(NativePlatformFailure),
    FirstFrame(ExecutableFirstFrameFailure),
    NativeInputReachability(ExecutableNativeInputReachabilityFailure),
    QueryCurrent(crate::adjudication::ExecutableQueryCurrentFailure),
    VisualIdentity(ExecutableVisualIdentityFailure),
    SourceAction(PulseSourceActionFailure),
    WatchedObservation(WatchedPulseObservationFailure),
    Replacement(ExecutableReplacementFailure),
    SchemaTransition(ExecutableSchemaTransitionFailure),
    Preservation(ExecutablePredecessorPreservationFailure),
    IntentJourney(crate::product_process::PlatformPulseIntentJourneyFailure),
    PortalJourney(crate::product_process::PlatformPulsePortalJourneyFailure),
    ProcessExit(PlatformPulseProcessExitFailure),
    NativeCloseEvidence(PlatformPulseNativeCloseEvidenceFailure),
    Cleanup(ExecutableLifecycleCleanupFailure),
    Quiescence(PlatformPulseQuiescenceFailure),
}

#[derive(Debug)]
pub(crate) struct PulseExecutableWorldFailureReport {
    primary: Box<PulseExecutableWorldFailure>,
    teardown: Box<ExecutableWorldFailureTeardown>,
    artifact: Box<Result<RetainedFailureArtifact, FailureArtifactFailure>>,
}

const _: () = assert!(std::mem::size_of::<PulseExecutableWorldFailureReport>() <= 64);

#[derive(Debug)]
pub(crate) enum ExecutableWorldFailureTeardown {
    NoOwnedResources,
    InstallationOnly(InstallationOnlyFailureTeardown),
    Unbound(UnboundFailureTeardown),
    NativeBound(NativeBoundFailureTeardown),
}

#[derive(Debug)]
pub(crate) struct InstallationOnlyFailureTeardown {
    pub(super) installation:
        Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure>,
}

#[derive(Debug)]
pub(crate) struct UnboundFailureTeardown {
    pub(super) process: Result<EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure>,
    pub(super) lifecycle:
        Result<PlatformPulseLifecycleTeardownEvidence, PlatformPulseLifecycleTeardownFailure>,
    pub(super) installation:
        Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure>,
}

#[derive(Debug)]
pub(crate) struct NativeBoundFailureTeardown {
    pub(super) process: Result<EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure>,
    pub(super) lifecycle:
        Result<PlatformPulseLifecycleTeardownEvidence, PlatformPulseLifecycleTeardownFailure>,
    pub(super) native_window: Result<(), NativePlatformFailure>,
    pub(super) installation:
        Result<PulseInstallationCleanupEvidence, PulseInstallationCleanupFailure>,
}

impl PulseExecutableWorldFailureReport {
    pub(super) fn new(
        primary: PulseExecutableWorldFailure,
        teardown: ExecutableWorldFailureTeardown,
        artifact_inputs: FailureArtifactInputs,
    ) -> Self {
        let artifact = RetainedFailureArtifact::create(&primary, &teardown, artifact_inputs);
        Self {
            primary: Box::new(primary),
            teardown: Box::new(teardown),
            artifact: Box::new(artifact),
        }
    }

    pub(crate) fn primary(&self) -> &PulseExecutableWorldFailure {
        &self.primary
    }

    pub(crate) fn teardown(&self) -> &ExecutableWorldFailureTeardown {
        &self.teardown
    }

    pub(crate) fn artifact(&self) -> Result<&RetainedFailureArtifact, &FailureArtifactFailure> {
        self.artifact.as_ref().as_ref()
    }

    pub(crate) fn discard_artifact(
        self,
    ) -> Result<FailureArtifactDiscardEvidence, FailureArtifactFailure> {
        (*self.artifact).and_then(RetainedFailureArtifact::discard)
    }
}

impl ExecutableWorldFailureTeardown {
    pub(crate) fn all_owned_resources_released(&self) -> bool {
        match self {
            Self::NoOwnedResources => true,
            Self::InstallationOnly(teardown) => teardown.installation.is_ok(),
            Self::Unbound(teardown) => {
                teardown.process.is_ok()
                    && teardown.lifecycle.is_ok()
                    && teardown.installation.is_ok()
            }
            Self::NativeBound(teardown) => {
                teardown.process.is_ok()
                    && teardown.lifecycle.is_ok()
                    && teardown.native_window.is_ok()
                    && teardown.installation.is_ok()
            }
        }
    }

    pub(crate) fn forced_process_termination(&self) -> Option<bool> {
        match self {
            Self::Unbound(teardown) => teardown
                .process
                .as_ref()
                .ok()
                .map(|exit| exit.forced_termination()),
            Self::NativeBound(teardown) => teardown
                .process
                .as_ref()
                .ok()
                .map(|exit| exit.forced_termination()),
            Self::NoOwnedResources | Self::InstallationOnly(_) => None,
        }
    }

    pub(crate) fn lifecycle_reader_joined(&self) -> Option<bool> {
        match self {
            Self::Unbound(teardown) => teardown
                .lifecycle
                .as_ref()
                .ok()
                .map(|evidence| evidence.reader_joined()),
            Self::NativeBound(teardown) => teardown
                .lifecycle
                .as_ref()
                .ok()
                .map(|evidence| evidence.reader_joined()),
            Self::NoOwnedResources | Self::InstallationOnly(_) => None,
        }
    }

    pub(crate) fn discarded_lifecycle_envelopes(&self) -> Option<usize> {
        match self {
            Self::Unbound(teardown) => teardown
                .lifecycle
                .as_ref()
                .ok()
                .copied()
                .map(|evidence| evidence.discarded_envelopes()),
            Self::NativeBound(teardown) => teardown
                .lifecycle
                .as_ref()
                .ok()
                .copied()
                .map(|evidence| evidence.discarded_envelopes()),
            Self::NoOwnedResources | Self::InstallationOnly(_) => None,
        }
    }

    pub(crate) fn installation_removed(&self) -> Option<bool> {
        match self {
            Self::InstallationOnly(teardown) => teardown
                .installation
                .as_ref()
                .ok()
                .copied()
                .map(|evidence| evidence.removed_owned_root()),
            Self::Unbound(teardown) => teardown
                .installation
                .as_ref()
                .ok()
                .copied()
                .map(|evidence| evidence.removed_owned_root()),
            Self::NativeBound(teardown) => teardown
                .installation
                .as_ref()
                .ok()
                .copied()
                .map(|evidence| evidence.removed_owned_root()),
            Self::NoOwnedResources => None,
        }
    }
}

impl fmt::Display for PulseExecutableWorldFailureReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "primary: {}; failure teardown: {}; retained artifact: ",
            self.primary, self.teardown,
        )
        .and_then(|()| match self.artifact.as_ref() {
            Ok(artifact) => write!(
                formatter,
                "{} ({} bytes)",
                artifact.path().display(),
                artifact.retained_bytes()
            ),
            Err(failure) => write!(formatter, "failed({failure})"),
        })
    }
}

impl fmt::Display for PulseExecutableWorldFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Installation(failure) => write!(formatter, "installation: {failure}"),
            Self::InstallationCleanup(failure) => {
                write!(formatter, "installation cleanup: {failure}")
            }
            Self::Launch(failure) => write!(formatter, "product launch: {failure}"),
            Self::Lifecycle(failure) => write!(formatter, "lifecycle stream: {failure}"),
            Self::Liveness(failure) => write!(formatter, "process liveness: {failure}"),
            Self::Native(failure) => write!(formatter, "native platform: {failure}"),
            Self::FirstFrame(failure) => {
                write!(formatter, "first-frame adjudication: {failure}")
            }
            Self::NativeInputReachability(failure) => {
                write!(
                    formatter,
                    "native-input reachability adjudication: {failure}"
                )
            }
            Self::QueryCurrent(failure) => {
                write!(formatter, "Query-current adjudication: {failure}")
            }
            Self::VisualIdentity(failure) => {
                write!(formatter, "visual-identity adjudication: {failure}")
            }
            Self::SourceAction(failure) => write!(formatter, "source action: {failure}"),
            Self::WatchedObservation(failure) => {
                write!(formatter, "watched observation: {failure}")
            }
            Self::Replacement(failure) => {
                write!(formatter, "replacement adjudication: {failure}")
            }
            Self::SchemaTransition(failure) => {
                write!(formatter, "schema-transition adjudication: {failure}")
            }
            Self::Preservation(failure) => {
                write!(formatter, "predecessor preservation: {failure}")
            }
            Self::IntentJourney(failure) => write!(formatter, "intent journey: {failure}"),
            Self::PortalJourney(failure) => write!(formatter, "portal journey: {failure}"),
            Self::ProcessExit(failure) => write!(formatter, "process exit: {failure}"),
            Self::NativeCloseEvidence(failure) => {
                write!(formatter, "native close evidence: {failure}")
            }
            Self::Cleanup(failure) => write!(formatter, "lifecycle cleanup: {failure}"),
            Self::Quiescence(failure) => write!(formatter, "product quiescence: {failure}"),
        }
    }
}

impl fmt::Display for ExecutableWorldFailureTeardown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOwnedResources => formatter.write_str("no owned resources"),
            Self::InstallationOnly(teardown) => {
                write!(formatter, "installation={:?}", teardown.installation)
            }
            Self::Unbound(teardown) => write!(
                formatter,
                "process={:?}, lifecycle={:?}, installation={:?}",
                teardown.process, teardown.lifecycle, teardown.installation
            ),
            Self::NativeBound(teardown) => write!(
                formatter,
                "process={:?}, lifecycle={:?}, native_window={:?}, installation={:?}",
                teardown.process, teardown.lifecycle, teardown.native_window, teardown.installation
            ),
        }
    }
}
