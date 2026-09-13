use std::time::Instant;

use crate::adjudication::{
    adjudicate_default_wrapping_text, adjudicate_query_current, adjudicate_visual_retirement,
    adjudicate_visual_snapshot,
};
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::source_delta::{QueryStatusV1, QueryStatusV2};

use super::watched_native_observation::observe_watched_native_until_schema_posture_changes;
use super::{
    await_watched_observation, AwaitingQueryCurrent, ComparisonBasisRefreshed, FirstCurrent,
    InitialBlue, NativeInputReached, OverlayCleared, Published, PulseExecutableWorld, QueryCurrent,
    SecondCurrent, SecondQueryCurrent, WatchedPulseTransition,
};

struct QueryExpectation<'value> {
    value: &'value str,
    owner_order: u64,
    requires_wrapping_text: bool,
    deadline: Instant,
}

impl PulseExecutableWorld<Published<NativeInputReached<InitialBlue>>> {
    pub(crate) fn publish_first_query_value(
        self,
        value: QueryStatusV1,
    ) -> Result<
        PulseExecutableWorld<AwaitingQueryCurrent<NativeInputReached<InitialBlue>, QueryStatusV1>>,
        PulseExecutableWorldFailureReport,
    > {
        apply_query(self, value, QueryStatusV1::apply)
    }
}

impl PulseExecutableWorld<Published<OverlayCleared<FirstCurrent>>> {
    pub(crate) fn publish_second_query_value(
        self,
        value: QueryStatusV2,
    ) -> Result<
        PulseExecutableWorld<AwaitingQueryCurrent<OverlayCleared<FirstCurrent>, QueryStatusV2>>,
        PulseExecutableWorldFailureReport,
    > {
        apply_query(self, value, QueryStatusV2::apply)
    }
}

impl PulseExecutableWorld<AwaitingQueryCurrent<NativeInputReached<InitialBlue>, QueryStatusV1>> {
    pub(crate) fn await_first_query_value(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<FirstCurrent>>, PulseExecutableWorldFailureReport>
    {
        await_query(
            self,
            QueryExpectation {
                value: QueryStatusV1::VALUE,
                owner_order: 2,
                requires_wrapping_text: false,
                deadline,
            },
            |prior| prior.prior.evidence.pixels(),
        )
    }
}

impl PulseExecutableWorld<AwaitingQueryCurrent<OverlayCleared<FirstCurrent>, QueryStatusV2>> {
    pub(crate) fn await_second_query_value(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<SecondCurrent>>, PulseExecutableWorldFailureReport>
    {
        let current = await_query(
            self,
            QueryExpectation {
                value: QueryStatusV2::VALUE,
                owner_order: 5,
                requires_wrapping_text: true,
                deadline,
            },
            |prior| prior.overlay.trace.snapshot.prior.evidence.pixels(),
        )?;
        await_comparison_basis_refresh(current, deadline)
    }
}

fn await_comparison_basis_refresh(
    current: PulseExecutableWorld<Published<SecondQueryCurrent>>,
    deadline: Instant,
) -> Result<PulseExecutableWorld<Published<SecondCurrent>>, PulseExecutableWorldFailureReport> {
    let Published {
        mut world,
        stage: current,
    } = current.state;
    let result = (|| {
        let retirement = await_watched_observation(
            &mut world.process,
            &mut world.lifecycle,
            WatchedPulseTransition::VisualSnapshotRetired,
            deadline,
        )
        .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
        let refreshed = await_watched_observation(
            &mut world.process,
            &mut world.lifecycle,
            WatchedPulseTransition::VisualSnapshot,
            deadline,
        )
        .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
        let frame = current.evidence.publication().frame().diagnostic_value();
        let expected_retirement = current.evidence.published_sequence().saturating_add(1);
        let retirement = adjudicate_visual_retirement(
            retirement,
            current.prior.snapshot_evidence(),
            frame,
            expected_retirement,
        )
        .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
        let snapshot =
            adjudicate_visual_snapshot(refreshed, frame, retirement.sequence().saturating_add(1))
                .map_err(PulseExecutableWorldFailure::VisualIdentity)?;
        Ok((retirement, snapshot))
    })();
    match result {
        Ok((retirement, snapshot)) => Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: ComparisonBasisRefreshed {
                    prior: current,
                    retirement,
                    snapshot,
                },
            },
        }),
        Err(primary) => Err(teardown_native_bound_world(
            primary,
            world.into_failure_resources(),
        )),
    }
}

fn apply_query<Stage, Kind>(
    world: PulseExecutableWorld<Published<Stage>>,
    value: Kind,
    apply: fn(
        Kind,
        &crate::installation::IsolatedPulseInstallation,
    ) -> Result<
        crate::source_delta::AppliedPulseSourceDelta<Kind>,
        crate::source_delta::PulseSourceActionFailure,
    >,
) -> Result<
    PulseExecutableWorld<AwaitingQueryCurrent<Stage, Kind>>,
    PulseExecutableWorldFailureReport,
> {
    let Published {
        world: native,
        stage,
    } = world.state;
    let action = match apply(value, &native.installation) {
        Ok(action) => action,
        Err(primary) => {
            return Err(teardown_native_bound_world(
                PulseExecutableWorldFailure::SourceAction(primary),
                native.into_failure_resources(),
            ))
        }
    };
    Ok(PulseExecutableWorld {
        state: AwaitingQueryCurrent {
            world: native,
            prior: stage,
            action,
        },
    })
}

fn await_query<Stage, Kind>(
    world: PulseExecutableWorld<AwaitingQueryCurrent<Stage, Kind>>,
    expectation: QueryExpectation<'_>,
    predecessor: impl FnOnce(&Stage) -> &crate::external_observation::NativeClientPixelCapture,
) -> Result<
    PulseExecutableWorld<Published<QueryCurrent<Stage, Kind>>>,
    PulseExecutableWorldFailureReport,
> {
    let AwaitingQueryCurrent {
        mut world,
        prior,
        action,
    } = world.state;
    let result = (|| {
        let issued = await_watched_observation(
            &mut world.process,
            &mut world.lifecycle,
            WatchedPulseTransition::QueryProjectionIssued,
            expectation.deadline,
        )
        .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
        let published = await_watched_observation(
            &mut world.process,
            &mut world.lifecycle,
            WatchedPulseTransition::QueryProjectionPublished,
            expectation.deadline,
        )
        .map_err(PulseExecutableWorldFailure::WatchedObservation)?;
        let predecessor = predecessor(&prior);
        let native = observe_watched_native_until_schema_posture_changes(
            &mut world,
            predecessor,
            expectation.deadline,
            "Query-current pixels",
        )?;
        let evidence = adjudicate_query_current(
            issued,
            published,
            expectation.value,
            expectation.owner_order,
            native.client,
            native.pixels,
            predecessor.rgba(),
        )
        .map_err(PulseExecutableWorldFailure::QueryCurrent)?;
        if expectation.requires_wrapping_text {
            adjudicate_default_wrapping_text(evidence.pixels())
                .map_err(crate::adjudication::ExecutableQueryCurrentFailure::WrappingText)
                .map_err(PulseExecutableWorldFailure::QueryCurrent)?;
        }
        Ok(evidence)
    })();
    match result {
        Ok(evidence) => Ok(PulseExecutableWorld {
            state: Published {
                world,
                stage: QueryCurrent {
                    prior,
                    action,
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
