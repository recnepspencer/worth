use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope;

use crate::adjudication::{
    adjudicate_first_frame, CausalFirstFrameObservationSet, ExecutableFirstFrameEvidence,
};
use crate::external_observation::{begin_stable_process_liveness, PlatformPulseLifecycleStream};
use crate::failure_teardown::{
    teardown_native_bound_world, teardown_unbound_world, PulseExecutableWorldFailure,
    PulseExecutableWorldFailureReport, UnboundFailureWorldResources,
};
use crate::native_platform::{
    CertifiedNativePlatform, CertifiedProcessBoundNativeClientArea, NativePlatformContract,
};

use super::{
    AwaitingFirstFrame, InitialBlue, LivePlatformPulseProcess, NativeBoundExecutableWorld,
    Published, PulseExecutableWorld,
};

struct BoundFirstFrameWorld {
    process_started: PlatformPulseLifecycleObservationEnvelope,
    pending_issued: PlatformPulseLifecycleObservationEnvelope,
    first_frame: PlatformPulseLifecycleObservationEnvelope,
    pending_published: PlatformPulseLifecycleObservationEnvelope,
    platform: CertifiedNativePlatform,
    native_client: CertifiedProcessBoundNativeClientArea,
    launch_to_first_publication: Duration,
}

impl PulseExecutableWorld<AwaitingFirstFrame> {
    pub(crate) fn await_first_frame(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<InitialBlue>>, PulseExecutableWorldFailureReport>
    {
        let AwaitingFirstFrame {
            installation,
            mut process,
            mut lifecycle,
            launch_started,
        } = self.state;
        let bound =
            match bind_first_frame_world(&mut process, &mut lifecycle, launch_started, deadline) {
                Ok(bound) => bound,
                Err(primary) => {
                    return Err(teardown_unbound_world(
                        primary,
                        UnboundFailureWorldResources::new(installation, process, lifecycle),
                    ))
                }
            };
        let evidence = match adjudicate_bound_first_frame(&mut process, &bound, deadline) {
            Ok(evidence) => evidence,
            Err(primary) => {
                return Err(teardown_native_bound_world(
                    primary,
                    UnboundFailureWorldResources::new(installation, process, lifecycle)
                        .bind_native(bound.platform, bound.native_client),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world: NativeBoundExecutableWorld {
                    installation,
                    process,
                    lifecycle,
                    journey_started: launch_started,
                    platform: bound.platform,
                    native_client: bound.native_client,
                },
                stage: InitialBlue {
                    evidence,
                    launch_to_first_publication: bound.launch_to_first_publication,
                },
            },
        })
    }
}

fn bind_first_frame_world(
    process: &mut LivePlatformPulseProcess,
    lifecycle: &mut PlatformPulseLifecycleStream,
    launch_started: Instant,
    deadline: Instant,
) -> Result<BoundFirstFrameWorld, PulseExecutableWorldFailure> {
    let process_started = lifecycle
        .next(deadline)
        .map_err(PulseExecutableWorldFailure::Lifecycle)?;
    let pending_issued = lifecycle
        .next(deadline)
        .map_err(PulseExecutableWorldFailure::Lifecycle)?;
    let first_frame = lifecycle
        .next(deadline)
        .map_err(PulseExecutableWorldFailure::Lifecycle)?;
    let pending_published = lifecycle
        .next(deadline)
        .map_err(PulseExecutableWorldFailure::Lifecycle)?;
    let launch_to_first_publication = launch_started.elapsed();
    let platform =
        CertifiedNativePlatform::certified().map_err(PulseExecutableWorldFailure::Native)?;
    let native_client = platform
        .bind_process_client_area(process.id(), deadline)
        .map_err(PulseExecutableWorldFailure::Native)?;
    Ok(BoundFirstFrameWorld {
        process_started,
        pending_issued,
        first_frame,
        pending_published,
        platform,
        native_client,
        launch_to_first_publication,
    })
}

fn adjudicate_bound_first_frame(
    process: &mut LivePlatformPulseProcess,
    bound: &BoundFirstFrameWorld,
    deadline: Instant,
) -> Result<ExecutableFirstFrameEvidence, PulseExecutableWorldFailure> {
    let liveness =
        begin_stable_process_liveness(process).map_err(PulseExecutableWorldFailure::Liveness)?;
    let client_area = bound
        .platform
        .observe_bound_client_area(&bound.native_client)
        .map_err(PulseExecutableWorldFailure::Native)?;
    let pixels = loop {
        let pixels = bound
            .platform
            .capture_client_area(&bound.native_client)
            .map_err(PulseExecutableWorldFailure::Native)?;
        if crate::adjudication::adjudicate_native_color(
            &pixels,
            crate::adjudication::ExpectedNativeColor::Blue,
        )
        .is_ok()
        {
            break pixels;
        }
        if Instant::now() >= deadline {
            return Err(PulseExecutableWorldFailure::Native(
                crate::native_platform::NativePlatformFailure::ClientPixelDeadline(
                    "first-frame-visible",
                ),
            ));
        }
        if process
            .observed_exit()
            .map_err(PulseExecutableWorldFailure::Launch)?
            .is_some()
        {
            return Err(PulseExecutableWorldFailure::Native(
                crate::native_platform::NativePlatformFailure::ClientPixelDeadline(
                    "first-frame-process-exited",
                ),
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let liveness = liveness
        .finish(process)
        .map_err(PulseExecutableWorldFailure::Liveness)?;
    let causal = CausalFirstFrameObservationSet::new(
        process.id(),
        bound.process_started.clone(),
        bound.pending_issued.clone(),
        bound.first_frame.clone(),
        bound.pending_published.clone(),
    );
    adjudicate_first_frame(causal.join_native(client_area, liveness, pixels))
        .map_err(PulseExecutableWorldFailure::FirstFrame)
}
