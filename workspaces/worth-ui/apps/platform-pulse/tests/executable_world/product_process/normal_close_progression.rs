use std::time::Instant;

use worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope;

use crate::adjudication::{adjudicate_lifecycle_cleanup, CausalLifecycleCleanupObservationSet};
use crate::external_observation::{
    NormalNativeCloseRequestObservation, PlatformPulseLifecycleStream,
};
use crate::failure_teardown::{
    report_without_owned_resources, teardown_native_bound_world, PulseExecutableWorldFailure,
    PulseExecutableWorldFailureReport, UnboundFailureWorldResources,
};
use crate::installation::{IsolatedPulseInstallation, PulseInstallationCleanupEvidence};
use crate::native_platform::{
    CertifiedNativePlatform, CertifiedProcessBoundNativeClientArea, NativePlatformContract,
};

use super::{
    Closed, LivePlatformPulseProcess, Published, PulseExecutableWorld, SuccessfulPlatformPulseExit,
};

struct PublishedNormalCloseWorld {
    installation: IsolatedPulseInstallation,
    process: LivePlatformPulseProcess,
    lifecycle: PlatformPulseLifecycleStream,
    platform: CertifiedNativePlatform,
    native_client: CertifiedProcessBoundNativeClientArea,
}

struct NormalCloseObservationSet {
    process_id: u32,
    close_request: NormalNativeCloseRequestObservation,
    shutdown_envelope: PlatformPulseLifecycleObservationEnvelope,
    lifecycle_measurement: crate::external_observation::LifecycleStreamMeasurement,
    lifecycle_envelopes: Vec<PlatformPulseLifecycleObservationEnvelope>,
    successful_exit: SuccessfulPlatformPulseExit,
    native_close_evidence: super::PlatformPulseNativeCloseEvidence,
    installation_cleanup: PulseInstallationCleanupEvidence,
}

impl<Stage> PulseExecutableWorld<Published<Stage>> {
    pub(crate) fn close_native_window(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Closed>, PulseExecutableWorldFailureReport> {
        let mut world = PublishedNormalCloseWorld::from_published(self.state);
        let observations = match world.complete_normal_close(deadline) {
            Ok(observations) => observations,
            Err(primary) => {
                return Err(teardown_native_bound_world(
                    primary,
                    world.into_failure_resources(),
                ))
            }
        };
        let causal = CausalLifecycleCleanupObservationSet::new(
            observations.process_id,
            observations.close_request,
            observations.shutdown_envelope,
            observations.lifecycle_measurement,
            observations.lifecycle_envelopes,
        );
        let evidence = adjudicate_lifecycle_cleanup(causal.join_resource_disposition(
            observations.successful_exit,
            observations.native_close_evidence,
            observations.installation_cleanup,
        ))
        .map_err(|failure| {
            report_without_owned_resources(PulseExecutableWorldFailure::Cleanup(failure))
        })?;
        Ok(PulseExecutableWorld {
            state: Closed { evidence },
        })
    }
}

impl PublishedNormalCloseWorld {
    fn from_published<Stage>(published: Published<Stage>) -> Self {
        let world = published.world;
        Self {
            installation: world.installation,
            process: world.process,
            lifecycle: world.lifecycle,
            platform: world.platform,
            native_client: world.native_client,
        }
    }

    fn complete_normal_close(
        &mut self,
        deadline: Instant,
    ) -> Result<NormalCloseObservationSet, PulseExecutableWorldFailure> {
        let process_id = self.process.id();
        let close_request = self.request_normal_close()?;
        let shutdown_envelope = self.await_shutdown(deadline)?;
        let successful_exit = self.await_successful_exit(deadline)?;
        self.settle_lifecycle_reader(deadline)?;
        let lifecycle_measurement = self.lifecycle.measurement();
        let lifecycle_envelopes = self.lifecycle.accepted_envelopes().to_vec();
        self.require_window_release(process_id)?;
        let native_close_evidence =
            super::PlatformPulseNativeCloseEvidence::read(self.installation.source_root())
                .map_err(PulseExecutableWorldFailure::NativeCloseEvidence)?;
        let installation_cleanup = self.cleanup_installation()?;
        Ok(NormalCloseObservationSet {
            process_id,
            close_request,
            shutdown_envelope,
            lifecycle_measurement,
            lifecycle_envelopes,
            successful_exit,
            native_close_evidence,
            installation_cleanup,
        })
    }

    fn request_normal_close(
        &self,
    ) -> Result<NormalNativeCloseRequestObservation, PulseExecutableWorldFailure> {
        self.platform
            .request_normal_close(&self.native_client)
            .map_err(PulseExecutableWorldFailure::Native)
    }

    fn await_shutdown(
        &mut self,
        deadline: Instant,
    ) -> Result<PlatformPulseLifecycleObservationEnvelope, PulseExecutableWorldFailure> {
        self.lifecycle
            .next(deadline)
            .map_err(PulseExecutableWorldFailure::Lifecycle)
    }

    fn await_successful_exit(
        &mut self,
        deadline: Instant,
    ) -> Result<SuccessfulPlatformPulseExit, PulseExecutableWorldFailure> {
        SuccessfulPlatformPulseExit::wait(&mut self.process, deadline)
            .map_err(PulseExecutableWorldFailure::ProcessExit)
    }

    fn settle_lifecycle_reader(
        &mut self,
        deadline: Instant,
    ) -> Result<(), PulseExecutableWorldFailure> {
        self.lifecycle
            .finish(deadline)
            .map_err(PulseExecutableWorldFailure::Lifecycle)
    }

    fn require_window_release(&self, process_id: u32) -> Result<(), PulseExecutableWorldFailure> {
        self.platform
            .verify_process_window_released(process_id)
            .map_err(PulseExecutableWorldFailure::Native)
    }

    fn cleanup_installation(
        &mut self,
    ) -> Result<PulseInstallationCleanupEvidence, PulseExecutableWorldFailure> {
        self.installation
            .close()
            .map_err(PulseExecutableWorldFailure::InstallationCleanup)
    }

    fn into_failure_resources(self) -> crate::failure_teardown::NativeBoundFailureWorldResources {
        UnboundFailureWorldResources::new(self.installation, self.process, self.lifecycle)
            .bind_native(self.platform, self.native_client)
    }
}
