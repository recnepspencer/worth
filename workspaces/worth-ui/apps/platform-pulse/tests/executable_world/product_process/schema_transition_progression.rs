use std::time::Instant;

use crate::adjudication::{
    adjudicate_replacement, adjudicate_schema_transition, adjudicate_successor_visual_snapshot,
    adjudicate_visual_comparison, adjudicate_visual_retirement, adjudicate_visual_snapshot,
    require_replacement_lifecycle, CausalReplacementObservationSet, ExpectedNativeColor,
    ExpectedSchemaTransition, ReplacementExpectation,
};
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};

use super::watched_native_observation::{
    observe_watched_native_until_schema_posture_changes,
    observe_watched_native_until_schema_posture_matches,
};
use super::{
    await_watched_observation, AwaitingSchemaStop, AwaitingStatusRecovery, FinalRecovered,
    NativeBoundExecutableWorld, Published, PulseExecutableWorld, SchemaStopped,
    WatchedPulseTransition,
};

impl PulseExecutableWorld<AwaitingSchemaStop> {
    pub(crate) fn await_schema_stopped(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<SchemaStopped>>, PulseExecutableWorldFailureReport>
    {
        let AwaitingSchemaStop {
            mut world,
            recovered,
            action,
        } = self.state;
        let result = (|| {
            let envelope = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::RevisionSchemaStopped,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            require_replacement_lifecycle(&envelope)
                .map_err(PulseExecutableWorldFailure::Replacement)?;
            let successor_snapshot = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::VisualSuccessorSnapshot,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            let comparison = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::VisualComparison,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            let retirement = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::VisualSnapshotRetired,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            let refreshed_snapshot = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::VisualSnapshot,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            let native = observe_watched_native_until_schema_posture_changes(
                &mut world,
                recovered.evidence.pixels(),
                deadline,
                "revision schema stopped posture",
            )?;
            let causal = CausalReplacementObservationSet::new(
                action,
                recovered.evidence.identity().clone(),
                envelope,
                ReplacementExpectation::revision_schema(),
            );
            let replacement = adjudicate_replacement(causal.join_native(
                native.client,
                native.liveness,
                native.pixels,
                ExpectedNativeColor::Blue,
            ))
            .map_err(PulseExecutableWorldFailure::Replacement)?;
            let successor_snapshot = adjudicate_successor_visual_snapshot(
                successor_snapshot,
                replacement
                    .replacement()
                    .successor_frame()
                    .diagnostic_value(),
                replacement.sequence().saturating_add(1),
            )
            .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
            let comparison = adjudicate_visual_comparison(
                comparison,
                &recovered.rebase_snapshot,
                &successor_snapshot,
                successor_snapshot.sequence().saturating_add(1),
            )
            .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
            let retirement = adjudicate_visual_retirement(
                retirement,
                &recovered.rebase_snapshot,
                replacement
                    .replacement()
                    .successor_frame()
                    .diagnostic_value(),
                comparison.sequence().saturating_add(1),
            )
            .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
            let worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservation::VisualSnapshotCaptured(
                observed_snapshot,
            ) = refreshed_snapshot.outcome()
            else {
                return Err(PulseExecutableWorldFailure::VisualIdentity(
                    crate::adjudication::ExecutableVisualIdentityFailure::WrongEvent(
                        "visual snapshot captured",
                    ),
                ));
            };
            let rebind_frame = replacement
                .replacement()
                .successor_frame()
                .diagnostic_value();
            let refresh_frame = observed_snapshot.affinity().frame();
            if refresh_frame <= rebind_frame {
                return Err(PulseExecutableWorldFailure::VisualIdentity(
                    crate::adjudication::ExecutableVisualIdentityFailure::RefreshDidNotAdvanceContent {
                        rebind_frame,
                        refresh_frame,
                    },
                ));
            }
            let refreshed_snapshot = adjudicate_visual_snapshot(
                refreshed_snapshot,
                refresh_frame,
                retirement.sequence().saturating_add(1),
            )
            .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
            let evidence = adjudicate_schema_transition(
                replacement,
                recovered.evidence.pixels(),
                None,
                recovered
                    .preserved
                    .green
                    .initial
                    .prior
                    .evidence
                    .projection(),
                ExpectedSchemaTransition::Stopped,
            )
            .map_err(PulseExecutableWorldFailure::SchemaTransition)?;
            Ok((evidence, retirement, refreshed_snapshot))
        })();
        let (evidence, retirement, snapshot) = match result {
            Ok(evidence) => evidence,
            Err(primary) => return Err(teardown(world, primary)),
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: SchemaStopped {
                    recovered,
                    evidence,
                    retirement,
                    snapshot,
                },
            },
        })
    }
}

impl PulseExecutableWorld<AwaitingStatusRecovery> {
    pub(crate) fn await_status_recovered(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<FinalRecovered>>, PulseExecutableWorldFailureReport>
    {
        let AwaitingStatusRecovery {
            mut world,
            stopped,
            action,
        } = self.state;
        let result = (|| {
            let envelope = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::StatusSchemaRecovered,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            require_replacement_lifecycle(&envelope)
                .map_err(PulseExecutableWorldFailure::Replacement)?;
            let retirement = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::VisualSnapshotRetired,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            let rebase_snapshot = await_watched_observation(
                &mut world.process,
                &mut world.lifecycle,
                WatchedPulseTransition::VisualSnapshot,
                deadline,
            )
            .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
            let native = observe_watched_native_until_schema_posture_matches(
                &mut world,
                stopped.recovered.evidence.pixels(),
                deadline,
                "canonical current recovery",
            )?;
            let causal = CausalReplacementObservationSet::new(
                action,
                stopped.evidence.replacement().identity().clone(),
                envelope,
                ReplacementExpectation::status_schema_recovery(
                    stopped
                        .recovered
                        .preserved
                        .green
                        .initial
                        .canonical_source_digest(),
                ),
            );
            let replacement = adjudicate_replacement(causal.join_native(
                native.client,
                native.liveness,
                native.pixels,
                ExpectedNativeColor::Blue,
            ))
            .map_err(PulseExecutableWorldFailure::Replacement)?;
            let worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservation::VisualSnapshotRetired(
                observed_retirement,
            ) = retirement.outcome()
            else {
                return Err(PulseExecutableWorldFailure::VisualIdentity(
                    crate::adjudication::ExecutableVisualIdentityFailure::WrongEvent(
                        "visual snapshot retired",
                    ),
                ));
            };
            let rebind_frame = replacement
                .replacement()
                .successor_frame()
                .diagnostic_value();
            let refresh_frame = observed_retirement.successor_frame();
            if refresh_frame <= rebind_frame {
                return Err(PulseExecutableWorldFailure::VisualIdentity(
                    crate::adjudication::ExecutableVisualIdentityFailure::RefreshDidNotAdvanceContent {
                        rebind_frame,
                        refresh_frame,
                    },
                ));
            }
            let retirement = adjudicate_visual_retirement(
                retirement,
                &stopped.snapshot,
                refresh_frame,
                replacement.sequence().saturating_add(1),
            )
            .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
            let rebase_snapshot = adjudicate_visual_snapshot(
                rebase_snapshot,
                refresh_frame,
                retirement.sequence().saturating_add(1),
            )
            .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
            let evidence = adjudicate_schema_transition(
                replacement,
                stopped.evidence.replacement().pixels(),
                Some(stopped.recovered.evidence.pixels()),
                stopped.evidence.query_basis(),
                ExpectedSchemaTransition::Recovered,
            )
            .map_err(PulseExecutableWorldFailure::SchemaTransition)?;
            Ok((evidence, rebase_snapshot))
        })();
        let (evidence, rebase_snapshot) = match result {
            Ok(evidence) => evidence,
            Err(primary) => return Err(teardown(world, primary)),
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: FinalRecovered {
                    stopped,
                    evidence,
                    rebase_snapshot,
                },
            },
        })
    }
}

fn teardown(
    world: NativeBoundExecutableWorld,
    primary: PulseExecutableWorldFailure,
) -> PulseExecutableWorldFailureReport {
    teardown_native_bound_world(primary, world.into_failure_resources())
}
