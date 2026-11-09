//! Public history and explicit export owners retain the actual event-page refusal.
use super::lineage_sources::EventFixture;
use worth_relational::facade::{
    change_source::RelationalObservationReadDenial,
    durability::*,
    history::{BranchId, LineageAspectHistory},
    lineage::*,
    transactions::RelationalExecutionDenialCause as Execution,
};
#[derive(Clone, Copy, Debug)]
pub(super) enum EventDoor {
    Historical,
    Record,
    Observation,
    Aspect,
    AspectTrace,
    AccessAspect,
    AccessAspectTrace,
    MemoryCheckpoint,
    EmbeddedCheckpoint,
}
impl EventDoor {
    pub(super) const ALL: [Self; 9] = [
        Self::Historical,
        Self::Record,
        Self::Observation,
        Self::Aspect,
        Self::AspectTrace,
        Self::AccessAspect,
        Self::AccessAspectTrace,
        Self::MemoryCheckpoint,
        Self::EmbeddedCheckpoint,
    ];
    pub(super) fn read(
        self,
        fixture: &EventFixture,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<usize, Execution> {
        let runtime = &fixture.runtime;
        let branch = BranchId("main".into());
        match self {
            Self::Historical => runtime
                .discover_authored_lineage_history(
                    execution,
                    &runtime.main_branch_identity(),
                    fixture.seed,
                )
                .map_err(execution_denial)
                .map(|resolution| verify_resolution(fixture, resolution)),
            Self::Record => runtime
                .discover_authored_record_history(
                    execution,
                    &runtime.main_branch_identity(),
                    fixture.original,
                )
                .map_err(execution_denial)
                .map(|resolution| {
                    verify_resolution(
                        fixture,
                        resolution.expect("the original record has a real lineage"),
                    )
                }),
            Self::Observation => {
                let (_, basis) = runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap();
                runtime
                    .record_history_at_observation(
                        execution,
                        &basis.observation(),
                        fixture.original,
                    )
                    .map(|resolution| {
                        verify_resolution(
                            fixture,
                            resolution.expect("selected ancestry contains the original lineage"),
                        )
                    })
                    .map_err(|denial| match denial {
                        RelationalObservationReadDenial::Execution(cause) => cause,
                        other => panic!("the observation belongs to this runtime: {other:?}"),
                    })
            }
            Self::Aspect => runtime
                .history()
                .lineage_entity_aspect_history(execution, &branch, fixture.seed, None)
                .map(|history| verify_aspects(fixture, history.expect("actual lineage history"))),
            Self::AspectTrace => runtime
                .history()
                .lineage_entity_aspect_history_with_trace(execution, &branch, fixture.seed, None)
                .map(|history| {
                    verify_aspects(fixture, history.history.expect("actual lineage history"))
                }),
            Self::AccessAspect => runtime
                .history()
                .lineage_entity_aspect_history(execution, &branch, fixture.seed, None)
                .map(|history| verify_aspects(fixture, history.expect("actual lineage history"))),
            Self::AccessAspectTrace => runtime
                .history()
                .lineage_entity_aspect_history_with_trace(execution, &branch, fixture.seed, None)
                .map(|history| {
                    verify_aspects(fixture, history.history.expect("actual lineage history"))
                }),
            Self::MemoryCheckpoint => runtime
                .durability_authority()
                .checkpoint(execution)
                .map(|checkpoint| {
                    checkpoint
                        .lineage
                        .digest_basis()
                        .published_lineage_event_count
                })
                .map_err(|stop| match stop {
                    RelationalRecoveryStop::Durability(error) => Execution::StorageUnavailable(
                        error
                            .storage_unavailable
                            .expect("typed lineage export refusal"),
                    ),
                    other => panic!("funded live capture request: {other:?}"),
                }),
            Self::EmbeddedCheckpoint => runtime
                .durability_authority()
                .native_checkpoint(execution, execution.allocation_policy())
                .map(|checkpoint| checkpoint.bytes().len())
                .map_err(|denial| match denial {
                    RelationalNativeCheckpointCaptureDenial::Durability(error) => {
                        Execution::StorageUnavailable(
                            error
                                .storage_unavailable
                                .expect("typed lineage member export refusal"),
                        )
                    }
                    other => panic!("funded live capture request: {other:?}"),
                }),
        }
    }
}
fn verify_resolution(fixture: &EventFixture, resolution: HistoricalLineageResolution) -> usize {
    assert_eq!(resolution.resolved, fixture.expected.resolved);
    assert_eq!(
        resolution.traversed_event_ids,
        fixture.expected.traversed_event_ids
    );
    resolution.traversed_event_ids.len()
}
fn verify_aspects(fixture: &EventFixture, history: LineageAspectHistory) -> usize {
    assert_eq!(
        history.traversed_event_ids,
        fixture.expected.traversed_event_ids
    );
    assert!(fixture
        .expected
        .resolved
        .iter()
        .all(|id| history.resolved_lineage_chain.contains(id)));
    assert!(!history.entries.is_empty());
    history.entries.len()
}

fn execution_denial(denial: RelationalLineageDiscoveryDenial) -> Execution {
    match denial {
        RelationalLineageDiscoveryDenial::Execution(cause) => cause,
        RelationalLineageDiscoveryDenial::Branch(cause) => {
            panic!("fixture owns its branch: {cause:?}")
        }
        RelationalLineageDiscoveryDenial::PublicationInFlight => {
            panic!("fixture has no concurrent writer")
        }
        RelationalLineageDiscoveryDenial::PerformedPublicationRequiresSettlement(commit) => {
            panic!("fixture settled {commit:?}")
        }
    }
}
