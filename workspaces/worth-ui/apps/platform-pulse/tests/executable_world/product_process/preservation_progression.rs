use std::time::Instant;

use crate::adjudication::{
    adjudicate_predecessor_preservation, CausalPredecessorPreservationObservationSet,
};
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};

use super::watched_native_observation::observe_watched_native;
use super::{
    await_watched_observation, AwaitingPreservation, PreservedPredecessor, PulseExecutableWorld,
    WatchedPulseTransition,
};

impl PulseExecutableWorld<AwaitingPreservation> {
    pub(crate) fn await_preserved_predecessor(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<PreservedPredecessor>, PulseExecutableWorldFailureReport> {
        let AwaitingPreservation {
            mut world,
            green,
            action,
        } = self.state;
        let envelope = match await_watched_observation(
            &mut world.process,
            &mut world.lifecycle,
            WatchedPulseTransition::MalformedPreservation,
            deadline,
        ) {
            Ok(envelope) => envelope,
            Err(failure) => {
                return Err(teardown_native_bound_world(
                    PulseExecutableWorldFailure::WatchedObservation(failure),
                    world.into_failure_resources(),
                ))
            }
        };
        let native = match observe_watched_native(&mut world) {
            Ok(native) => native,
            Err(primary) => {
                return Err(teardown_native_bound_world(
                    primary,
                    world.into_failure_resources(),
                ))
            }
        };
        let causal = CausalPredecessorPreservationObservationSet::new(
            action,
            green.evidence.identity().clone(),
            green.snapshot.snapshot().affinity().frame(),
            envelope,
        );
        let evidence = match adjudicate_predecessor_preservation(causal.join_native(
            native.client,
            native.liveness,
            native.pixels,
        )) {
            Ok(evidence) => evidence,
            Err(failure) => {
                return Err(teardown_native_bound_world(
                    PulseExecutableWorldFailure::Preservation(failure),
                    world.into_failure_resources(),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: PreservedPredecessor {
                world,
                green,
                evidence,
            },
        })
    }
}
