use std::time::Instant;

use worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope;

use crate::adjudication::{
    adjudicate_replacement, adjudicate_visual_retirement, adjudicate_visual_snapshot,
    CausalReplacementObservationSet, ExecutableReplacementEvidence,
    ExecutableVisualIdentityFailure, ExecutableVisualRetirementEvidence,
    ExecutableVisualSnapshotEvidence, ExpectedNativeColor, ReplacementExpectation,
};
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::source_delta::{
    AppliedPulseSourceDelta, CanonicalBlueRecoverySourceDelta, GreenPulseSourceDelta,
};

use super::watched_native_observation::{
    observe_watched_native, observe_watched_native_stable, WatchedNativeObservation,
};
use super::{
    await_watched_observation, AwaitingRecovery, AwaitingReplacement, GreenSuccessor,
    NativeBoundExecutableWorld, PreservedPredecessorEvidence, Published, PulseExecutableWorld,
    RecoveredBlue, SecondCurrent, WatchedPulseTransition,
};

struct WatchedReplacementObservation {
    envelope: PlatformPulseLifecycleObservationEnvelope,
    visual: WatchedReplacementVisualObservation,
    native: WatchedNativeObservation,
}

struct WatchedReplacementVisualObservation {
    retirement: Option<PlatformPulseLifecycleObservationEnvelope>,
    refreshed_snapshot: Option<PlatformPulseLifecycleObservationEnvelope>,
}

impl PulseExecutableWorld<AwaitingReplacement> {
    pub(crate) fn await_green_successor(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<GreenSuccessor>>, PulseExecutableWorldFailureReport>
    {
        let AwaitingReplacement {
            mut world,
            initial,
            action,
        } = self.state;
        let observed = match observe_replacement(
            &mut world,
            WatchedPulseTransition::GreenReplacement,
            deadline,
        ) {
            Ok(observed) => observed,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let (evidence, snapshot, retirement) = match green_evidence(action, &initial, observed) {
            Ok(evidence) => evidence,
            Err(primary) => return Err(teardown(world, primary)),
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: GreenSuccessor {
                    initial,
                    evidence,
                    snapshot,
                    retirement,
                },
            },
        })
    }
}

impl PulseExecutableWorld<AwaitingRecovery> {
    pub(crate) fn await_recovered_blue(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<RecoveredBlue>>, PulseExecutableWorldFailureReport>
    {
        let AwaitingRecovery {
            mut world,
            preserved,
            action,
        } = self.state;
        let observed = match observe_replacement(
            &mut world,
            WatchedPulseTransition::CanonicalBlueRecovery,
            deadline,
        ) {
            Ok(observed) => observed,
            Err(primary) => return Err(teardown(world, primary)),
        };
        let (evidence, rebase_snapshot) = match recovery_evidence(action, &preserved, observed) {
            Ok(evidence) => evidence,
            Err(failure) => return Err(teardown(world, failure)),
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: RecoveredBlue {
                    preserved,
                    evidence,
                    rebase_snapshot,
                },
            },
        })
    }
}

fn observe_replacement(
    world: &mut NativeBoundExecutableWorld,
    transition: WatchedPulseTransition,
    deadline: Instant,
) -> Result<WatchedReplacementObservation, PulseExecutableWorldFailure> {
    let envelope = await_watched_observation(
        &mut world.process,
        &mut world.lifecycle,
        transition,
        deadline,
    )
    .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
    let (retirement, refreshed_snapshot) = match transition {
        WatchedPulseTransition::GreenReplacement => (
            Some(
                await_watched_observation(
                    &mut world.process,
                    &mut world.lifecycle,
                    WatchedPulseTransition::VisualSnapshotRetired,
                    deadline,
                )
                .map_err(PulseExecutableWorldFailure::WatchedObservation)?,
            ),
            Some(
                await_watched_observation(
                    &mut world.process,
                    &mut world.lifecycle,
                    WatchedPulseTransition::VisualSnapshot,
                    deadline,
                )
                .map_err(PulseExecutableWorldFailure::WatchedObservation)?,
            ),
        ),
        WatchedPulseTransition::CanonicalBlueRecovery => (
            Some(
                await_watched_observation(
                    &mut world.process,
                    &mut world.lifecycle,
                    WatchedPulseTransition::VisualSnapshotRetired,
                    deadline,
                )
                .map_err(PulseExecutableWorldFailure::WatchedObservation)?,
            ),
            Some(
                await_watched_observation(
                    &mut world.process,
                    &mut world.lifecycle,
                    WatchedPulseTransition::VisualSnapshot,
                    deadline,
                )
                .map_err(PulseExecutableWorldFailure::WatchedObservation)?,
            ),
        ),
        _ => (None, None),
    };
    let native = match transition {
        WatchedPulseTransition::CanonicalBlueRecovery => {
            observe_watched_native_stable(world, deadline, "stable canonical blue recovery")?
        }
        _ => observe_watched_native(world)?,
    };
    Ok(WatchedReplacementObservation {
        envelope,
        visual: WatchedReplacementVisualObservation {
            retirement,
            refreshed_snapshot,
        },
        native,
    })
}

fn green_evidence(
    action: AppliedPulseSourceDelta<GreenPulseSourceDelta>,
    initial: &SecondCurrent,
    observed: WatchedReplacementObservation,
) -> Result<
    (
        ExecutableReplacementEvidence<GreenPulseSourceDelta>,
        ExecutableVisualSnapshotEvidence,
        ExecutableVisualRetirementEvidence,
    ),
    PulseExecutableWorldFailure,
> {
    let WatchedReplacementObservation {
        envelope,
        visual: observed_visual,
        native,
    } = observed;
    let visual = initial.visual();
    let causal = CausalReplacementObservationSet::new(
        action,
        visual
            .initial()
            .prior
            .first_frame_evidence()
            .published_identity(),
        envelope,
        ReplacementExpectation::green_successor(),
    );
    let evidence = adjudicate_replacement(causal.join_native(
        native.client,
        native.liveness,
        native.pixels,
        ExpectedNativeColor::Green,
    ))
    .map_err(PulseExecutableWorldFailure::Replacement)?;
    let (snapshot, retirement) = green_visual_evidence(initial, &evidence, observed_visual)?;
    Ok((evidence, snapshot, retirement))
}

fn green_visual_evidence(
    initial: &SecondCurrent,
    evidence: &ExecutableReplacementEvidence<GreenPulseSourceDelta>,
    observed: WatchedReplacementVisualObservation,
) -> Result<
    (
        ExecutableVisualSnapshotEvidence,
        ExecutableVisualRetirementEvidence,
    ),
    PulseExecutableWorldFailure,
> {
    let retirement = observed
        .retirement
        .ok_or(PulseExecutableWorldFailure::VisualIdentity(
            ExecutableVisualIdentityFailure::WrongEvent("visual snapshot retired"),
        ))?;
    let worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservation::VisualSnapshotRetired(
        observed_retirement,
    ) = retirement.outcome()
    else {
        return Err(PulseExecutableWorldFailure::VisualIdentity(
            ExecutableVisualIdentityFailure::WrongEvent("visual snapshot retired"),
        ));
    };
    let rebind_frame = evidence.replacement().successor_frame().diagnostic_value();
    let refresh_frame = observed_retirement.successor_frame();
    if refresh_frame <= rebind_frame {
        return Err(PulseExecutableWorldFailure::VisualIdentity(
            ExecutableVisualIdentityFailure::RefreshDidNotAdvanceContent {
                rebind_frame,
                refresh_frame,
            },
        ));
    }
    let retirement = adjudicate_visual_retirement(
        retirement,
        initial.snapshot_evidence(),
        refresh_frame,
        evidence.sequence().saturating_add(1),
    )
    .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
    let refreshed_snapshot =
        observed
            .refreshed_snapshot
            .ok_or(PulseExecutableWorldFailure::VisualIdentity(
                ExecutableVisualIdentityFailure::WrongEvent("refreshed visual snapshot"),
            ))?;
    let refreshed_snapshot = adjudicate_visual_snapshot(
        refreshed_snapshot,
        refresh_frame,
        retirement.sequence().saturating_add(1),
    )
    .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
    Ok((refreshed_snapshot, retirement))
}

fn recovery_evidence(
    action: AppliedPulseSourceDelta<CanonicalBlueRecoverySourceDelta>,
    preserved: &PreservedPredecessorEvidence,
    observed: WatchedReplacementObservation,
) -> Result<
    (
        ExecutableReplacementEvidence<CanonicalBlueRecoverySourceDelta>,
        crate::adjudication::ExecutableVisualSnapshotEvidence,
    ),
    PulseExecutableWorldFailure,
> {
    let canonical_digest = preserved.green.initial.canonical_source_digest();
    let causal = CausalReplacementObservationSet::new(
        action,
        preserved.evidence.identity().clone(),
        observed.envelope,
        ReplacementExpectation::canonical_recovery(canonical_digest),
    );
    let evidence = adjudicate_replacement(causal.join_native(
        observed.native.client,
        observed.native.liveness,
        observed.native.pixels,
        ExpectedNativeColor::Blue,
    ))
    .map_err(PulseExecutableWorldFailure::Replacement)?;
    let retirement =
        observed
            .visual
            .retirement
            .ok_or(PulseExecutableWorldFailure::VisualIdentity(
                ExecutableVisualIdentityFailure::WrongEvent("visual snapshot retired"),
            ))?;
    let worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservation::VisualSnapshotRetired(
        observed_retirement,
    ) = retirement.outcome()
    else {
        return Err(PulseExecutableWorldFailure::VisualIdentity(
            ExecutableVisualIdentityFailure::WrongEvent("visual snapshot retired"),
        ));
    };
    let rebind_frame = evidence.replacement().successor_frame().diagnostic_value();
    let refresh_frame = observed_retirement.successor_frame();
    if refresh_frame <= rebind_frame {
        return Err(PulseExecutableWorldFailure::VisualIdentity(
            ExecutableVisualIdentityFailure::RefreshDidNotAdvanceContent {
                rebind_frame,
                refresh_frame,
            },
        ));
    }
    let retirement = adjudicate_visual_retirement(
        retirement,
        &preserved.green.snapshot,
        refresh_frame,
        evidence.sequence().saturating_add(1),
    )
    .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
    let rebase_snapshot =
        observed
            .visual
            .refreshed_snapshot
            .ok_or(PulseExecutableWorldFailure::VisualIdentity(
                ExecutableVisualIdentityFailure::WrongEvent("rebase visual snapshot"),
            ))?;
    let rebase_snapshot = adjudicate_visual_snapshot(
        rebase_snapshot,
        refresh_frame,
        retirement.sequence().saturating_add(1),
    )
    .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
    Ok((evidence, rebase_snapshot))
}

fn teardown(
    world: NativeBoundExecutableWorld,
    primary: PulseExecutableWorldFailure,
) -> PulseExecutableWorldFailureReport {
    teardown_native_bound_world(primary, world.into_failure_resources())
}
