use std::time::Instant;

use crate::adjudication::{
    adjudicate_native_input_reachability, native_input_background_point,
    NativeInputFamilyObservation, NativeInputReachabilityObservationSet,
};
use crate::external_observation::NativeInputProbeKind;
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::native_platform::NativePlatformContract;

use super::watched_native_observation::observe_watched_native;
use super::{
    InitialBlue, NativeBoundExecutableWorld, NativeInputReached, Published, PulseExecutableWorld,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeInputCausalStep {
    PointerDelivered,
    PointerObserved,
    KeyboardDelivered,
    KeyboardObserved,
}

impl PulseExecutableWorld<Published<InitialBlue>> {
    pub(crate) fn reach_native_input(
        self,
        deadline: Instant,
    ) -> Result<
        PulseExecutableWorld<Published<NativeInputReached<InitialBlue>>>,
        PulseExecutableWorldFailureReport,
    > {
        self.reach_native_input_observed(deadline, |_| {})
    }

    pub(crate) fn reach_native_input_observed(
        self,
        deadline: Instant,
        mut observe_step: impl FnMut(NativeInputCausalStep),
    ) -> Result<
        PulseExecutableWorld<Published<NativeInputReached<InitialBlue>>>,
        PulseExecutableWorldFailureReport,
    > {
        let Published {
            mut world,
            stage: initial,
        } = self.state;
        let point = native_input_background_point(&initial.evidence)
            .map_err(PulseExecutableWorldFailure::NativeInputReachability);
        let result = point
            .and_then(|point| observe_native_input(&mut world, point, deadline, &mut observe_step))
            .and_then(|observations| {
                adjudicate_native_input_reachability(&initial.evidence, observations)
                    .map_err(PulseExecutableWorldFailure::NativeInputReachability)
            });
        match result {
            Ok(evidence) => Ok(PulseExecutableWorld {
                state: Published {
                    world,
                    stage: NativeInputReached {
                        prior: initial,
                        evidence,
                    },
                },
            }),
            Err(primary) => Err(teardown_native_bound_world(
                primary,
                world.into_failure_resources(),
            )),
        }
    }
}

fn observe_native_input(
    world: &mut NativeBoundExecutableWorld,
    pointer_point: crate::external_observation::NativeClientPixelPoint,
    deadline: Instant,
    observe_step: &mut impl FnMut(NativeInputCausalStep),
) -> Result<NativeInputReachabilityObservationSet, PulseExecutableWorldFailure> {
    let pointer = world
        .platform
        .deliver_pointer_activation(&world.native_client, pointer_point)
        .map_err(PulseExecutableWorldFailure::Native)?;
    observe_step(NativeInputCausalStep::PointerDelivered);
    let pointer = NativeInputFamilyObservation::new(
        pointer,
        world
            .lifecycle
            .next(deadline)
            .map_err(PulseExecutableWorldFailure::Lifecycle)?,
    );
    observe_step(NativeInputCausalStep::PointerObserved);
    let keyboard = world
        .platform
        .deliver_input_reachability_probe(&world.native_client, NativeInputProbeKind::Keyboard)
        .map_err(PulseExecutableWorldFailure::Native)?;
    observe_step(NativeInputCausalStep::KeyboardDelivered);
    let keyboard_routing_stop = world
        .lifecycle
        .next(deadline)
        .map_err(PulseExecutableWorldFailure::Lifecycle)?;
    let keyboard = NativeInputFamilyObservation::new(
        keyboard,
        world
            .lifecycle
            .next(deadline)
            .map_err(PulseExecutableWorldFailure::Lifecycle)?,
    );
    observe_step(NativeInputCausalStep::KeyboardObserved);
    let observed = observe_watched_native(world)?;
    Ok(NativeInputReachabilityObservationSet::new(
        pointer,
        keyboard_routing_stop,
        keyboard,
        observed.pixels,
    ))
}
