use std::time::Instant;

use crate::adjudication::{
    adjudicate_overlay_pixels, adjudicate_restored_pixels, adjudicate_visual_snapshot,
    adjudicate_visual_trace,
};
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};

use super::watched_native_observation::observe_watched_native;
use super::{
    await_watched_observation, FirstCurrent, IdentityTraced, NativeBoundExecutableWorld,
    OverlayCleared, OverlayPublished, Published, PulseExecutableWorld, SnapshotCaptured,
    WatchedPulseTransition,
};

impl PulseExecutableWorld<Published<FirstCurrent>> {
    pub(crate) fn await_visual_snapshot(
        self,
        deadline: Instant,
    ) -> Result<
        PulseExecutableWorld<Published<SnapshotCaptured<FirstCurrent>>>,
        PulseExecutableWorldFailureReport,
    > {
        let Published {
            mut world,
            stage: initial,
        } = self.state;
        let envelope = match await_visual_event(
            &mut world,
            WatchedPulseTransition::VisualSnapshot,
            deadline,
        ) {
            Ok(envelope) => envelope,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let frame = initial.evidence.publication().frame().diagnostic_value();
        let expected_sequence = initial.evidence.published_sequence().saturating_add(1);
        let evidence = match adjudicate_visual_snapshot(envelope, frame, expected_sequence) {
            Ok(evidence) => evidence,
            Err(failure) => {
                return Err(teardown(
                    world,
                    PulseExecutableWorldFailure::VisualIdentity(failure),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: SnapshotCaptured {
                    prior: initial,
                    evidence,
                },
            },
        })
    }
}

impl PulseExecutableWorld<Published<SnapshotCaptured<FirstCurrent>>> {
    pub(crate) fn await_identity_trace(
        self,
        deadline: Instant,
    ) -> Result<
        PulseExecutableWorld<Published<IdentityTraced<FirstCurrent>>>,
        PulseExecutableWorldFailureReport,
    > {
        let Published {
            mut world,
            stage: snapshot,
        } = self.state;
        let envelope = match await_visual_event(
            &mut world,
            WatchedPulseTransition::VisualIdentityTrace,
            deadline,
        ) {
            Ok(envelope) => envelope,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let evidence = match adjudicate_visual_trace(envelope, &snapshot.evidence) {
            Ok(evidence) => evidence,
            Err(failure) => {
                return Err(teardown(
                    world,
                    PulseExecutableWorldFailure::VisualIdentity(failure),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: IdentityTraced { snapshot, evidence },
            },
        })
    }
}

impl PulseExecutableWorld<Published<IdentityTraced<FirstCurrent>>> {
    pub(crate) fn await_overlay_published(
        self,
        deadline: Instant,
    ) -> Result<
        PulseExecutableWorld<Published<OverlayPublished<FirstCurrent>>>,
        PulseExecutableWorldFailureReport,
    > {
        let Published {
            mut world,
            stage: trace,
        } = self.state;
        let envelope = match await_visual_event(
            &mut world,
            WatchedPulseTransition::VisualOverlayPublished,
            deadline,
        ) {
            Ok(envelope) => envelope,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let native = match observe_watched_native(&mut world) {
            Ok(native) => native,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let evidence = match adjudicate_overlay_pixels(
            envelope,
            &trace.snapshot.evidence,
            &trace.evidence,
            world.process.id(),
            native.pixels,
        ) {
            Ok(evidence) => evidence,
            Err(failure) => {
                return Err(teardown(
                    world,
                    PulseExecutableWorldFailure::VisualIdentity(failure),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: OverlayPublished { trace, evidence },
            },
        })
    }
}

impl PulseExecutableWorld<Published<OverlayPublished<FirstCurrent>>> {
    pub(crate) fn await_overlay_cleared(
        self,
        deadline: Instant,
    ) -> Result<
        PulseExecutableWorld<Published<OverlayCleared<FirstCurrent>>>,
        PulseExecutableWorldFailureReport,
    > {
        let Published {
            mut world,
            stage: overlay,
        } = self.state;
        let envelope = match await_visual_event(
            &mut world,
            WatchedPulseTransition::VisualOverlayCleared,
            deadline,
        ) {
            Ok(envelope) => envelope,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let native = match observe_watched_native(&mut world) {
            Ok(native) => native,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let evidence = match adjudicate_restored_pixels(
            envelope,
            &overlay.evidence,
            world.process.id(),
            native.pixels,
        ) {
            Ok(evidence) => evidence,
            Err(failure) => {
                return Err(teardown(
                    world,
                    PulseExecutableWorldFailure::VisualIdentity(failure),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: OverlayCleared { overlay, evidence },
            },
        })
    }
}

fn await_visual_event(
    world: &mut NativeBoundExecutableWorld,
    transition: WatchedPulseTransition,
    deadline: Instant,
) -> Result<
    worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope,
    PulseExecutableWorldFailure,
> {
    await_watched_observation(
        &mut world.process,
        &mut world.lifecycle,
        transition,
        deadline,
    )
    .map_err(PulseExecutableWorldFailure::WatchedObservation)
}

fn teardown(
    world: NativeBoundExecutableWorld,
    primary: PulseExecutableWorldFailure,
) -> PulseExecutableWorldFailureReport {
    teardown_native_bound_world(primary, world.into_failure_resources())
}
