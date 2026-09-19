use std::time::{Duration, Instant};

use crate::external_observation::PlatformPulseLifecycleStream;
use crate::installation::IsolatedPulseInstallation;
use crate::native_platform::{
    NativePlatformContract, WindowsNativePlatform, WindowsProcessBoundNativeClientArea,
};
use crate::product_process::LivePlatformPulseProcess;

use super::report::{
    ExecutableWorldFailureTeardown, InstallationOnlyFailureTeardown, NativeBoundFailureTeardown,
    PulseExecutableWorldFailure, PulseExecutableWorldFailureReport, UnboundFailureTeardown,
};
use super::retained_artifact::{FailureArtifactInputs, FailureNativeCaptureInput};

const FAILURE_TEARDOWN_BUDGET: Duration = Duration::from_secs(5);

pub(crate) struct UnboundFailureWorldResources {
    installation: IsolatedPulseInstallation,
    process: LivePlatformPulseProcess,
    lifecycle: PlatformPulseLifecycleStream,
}

pub(crate) struct NativeBoundFailureWorldResources {
    unbound: UnboundFailureWorldResources,
    platform: WindowsNativePlatform,
    native_client: WindowsProcessBoundNativeClientArea,
}

pub(crate) fn report_without_owned_resources(
    primary: PulseExecutableWorldFailure,
) -> PulseExecutableWorldFailureReport {
    PulseExecutableWorldFailureReport::new(
        primary,
        ExecutableWorldFailureTeardown::NoOwnedResources,
        FailureArtifactInputs::none(),
    )
}

pub(crate) fn teardown_installed_world(
    primary: PulseExecutableWorldFailure,
    mut installation: IsolatedPulseInstallation,
) -> PulseExecutableWorldFailureReport {
    let source_snapshot = installation.failure_source_snapshot();
    let teardown = InstallationOnlyFailureTeardown {
        installation: installation.close(),
    };
    PulseExecutableWorldFailureReport::new(
        primary,
        ExecutableWorldFailureTeardown::InstallationOnly(teardown),
        FailureArtifactInputs {
            source_snapshot,
            lifecycle: None,
            native_capture: None,
        },
    )
}

pub(crate) fn teardown_unbound_world(
    primary: PulseExecutableWorldFailure,
    resources: UnboundFailureWorldResources,
) -> PulseExecutableWorldFailureReport {
    let UnboundFailureWorldResources {
        mut installation,
        mut process,
        lifecycle,
    } = resources;
    let deadline = Instant::now() + FAILURE_TEARDOWN_BUDGET;
    let source_snapshot = installation.failure_source_snapshot();
    let lifecycle_snapshot = lifecycle.failure_snapshot();
    let process = process.terminate_after_failure(deadline);
    let lifecycle = lifecycle.teardown_after_failure(deadline);
    let installation = installation.close();
    PulseExecutableWorldFailureReport::new(
        primary,
        ExecutableWorldFailureTeardown::Unbound(UnboundFailureTeardown {
            process,
            lifecycle,
            installation,
        }),
        FailureArtifactInputs {
            source_snapshot,
            lifecycle: Some(lifecycle_snapshot),
            native_capture: None,
        },
    )
}

pub(crate) fn teardown_native_bound_world(
    primary: PulseExecutableWorldFailure,
    resources: NativeBoundFailureWorldResources,
) -> PulseExecutableWorldFailureReport {
    let NativeBoundFailureWorldResources {
        unbound:
            UnboundFailureWorldResources {
                mut installation,
                mut process,
                lifecycle,
            },
        platform,
        native_client,
    } = resources;
    let process_id = process.id();
    let deadline = Instant::now() + FAILURE_TEARDOWN_BUDGET;
    let source_snapshot = installation.failure_source_snapshot();
    let lifecycle_snapshot = lifecycle.failure_snapshot();
    let native_capture = Some(match platform.capture_client_area(&native_client) {
        Ok(capture) => FailureNativeCaptureInput::Captured(capture),
        Err(failure) => FailureNativeCaptureInput::Failed(failure.to_string()),
    });
    let process = process.terminate_after_failure(deadline);
    let lifecycle = lifecycle.teardown_after_failure(deadline);
    let native_window = platform.verify_process_window_released(process_id);
    let installation = installation.close();
    PulseExecutableWorldFailureReport::new(
        primary,
        ExecutableWorldFailureTeardown::NativeBound(NativeBoundFailureTeardown {
            process,
            lifecycle,
            native_window,
            installation,
        }),
        FailureArtifactInputs {
            source_snapshot,
            lifecycle: Some(lifecycle_snapshot),
            native_capture,
        },
    )
}

impl UnboundFailureWorldResources {
    pub(crate) fn new(
        installation: IsolatedPulseInstallation,
        process: LivePlatformPulseProcess,
        lifecycle: PlatformPulseLifecycleStream,
    ) -> Self {
        Self {
            installation,
            process,
            lifecycle,
        }
    }

    pub(crate) fn bind_native(
        self,
        platform: WindowsNativePlatform,
        native_client: WindowsProcessBoundNativeClientArea,
    ) -> NativeBoundFailureWorldResources {
        NativeBoundFailureWorldResources {
            unbound: self,
            platform,
            native_client,
        }
    }
}
