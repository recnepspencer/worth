//! Every event-consuming door refuses an unavailable real replacement page.
use super::cold_lineage_event_fixture::{reopened, request, EventFixture};
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::history::data::{BranchId, LineageAspectHistory};
use crate::lineage::data::*;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[derive(Clone, Copy, Debug)]
pub(super) enum EventDoor {
    Historical,
    Record,
    Observation,
    Aspect,
    AspectTrace,
    AccessAspect,
    AccessAspectTrace,
    Graph,
    Divergence,
    MemoryCheckpoint,
    EmbeddedCheckpoint,
}
impl EventDoor {
    const ALL: [Self; 11] = [
        Self::Historical,
        Self::Record,
        Self::Observation,
        Self::Aspect,
        Self::AspectTrace,
        Self::AccessAspect,
        Self::AccessAspectTrace,
        Self::Graph,
        Self::Divergence,
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
            Self::Historical => runtime.lineage_access().resolve_historical_lineage(execution, request(fixture.seed))
                .map(|resolution| verify_resolution(fixture, resolution)),
            Self::Record => runtime.lineage_access().resolve_record_history(execution, RecordHistoryRequest {
                branch_id: branch, entity_id: fixture.original,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            }).map(|resolution| verify_resolution(fixture, resolution.expect("the original record has a real lineage"))),
            Self::Observation => {
                let (_, basis) = runtime.observe_branch(&runtime.main_branch_identity()).unwrap();
                runtime.record_history_at_observation(execution, &basis.observation(), fixture.original)
                    .map(|resolution| verify_resolution(fixture, resolution.expect("selected ancestry contains the original lineage")))
                    .map_err(|denial| match denial {
                        crate::change_source::RelationalObservationReadDenial::Execution(cause) => cause,
                        other => panic!("the observation belongs to this runtime: {other:?}"),
                    })
            }
            Self::Aspect => runtime.history().lineage_entity_aspect_history(execution, &branch, fixture.seed, None)
                .map(|history| verify_aspects(fixture, history.expect("actual lineage history"))),
            Self::AspectTrace => runtime.history().lineage_entity_aspect_history_with_trace(execution, &branch, fixture.seed, None)
                .map(|history| verify_aspects(fixture, history.history.expect("actual lineage history"))),
            Self::AccessAspect => runtime.lineage_access().entity_aspect_history(execution, request(fixture.seed), None)
                .map(|history| verify_aspects(fixture, history.expect("actual lineage history"))),
            Self::AccessAspectTrace => runtime.lineage_access().entity_aspect_history_with_trace(execution, request(fixture.seed), None)
                .map(|history| verify_aspects(fixture, history.history.expect("actual lineage history"))),
            Self::Graph => runtime.lineage_access().graph(execution, LineageGraphRequest {
                branch_id: branch, traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
            }).map(|graph| {
                for id in &fixture.expected.traversed_event_ids {
                    assert!(graph.events.iter().any(|event| event.event_id() == *id));
                }
                graph.events.len()
            }),
            Self::Divergence => runtime.lineage_access().divergence_between_branches(execution, LineageDivergenceRequest {
                left_branch: branch.clone(), right_branch: branch,
                traversal_basis: LineageDivergenceTraversalBasis::FullBranchGraphComparison,
            }).map(|divergence| divergence.metrics.left_event_count),
            Self::MemoryCheckpoint => runtime.durability_authority().checkpoint(execution)
                .map(|checkpoint| checkpoint.lineage.digest_basis().published_lineage_event_count)
                .map_err(|stop| match stop {
                    RelationalRecoveryStop::Durability(error) => Execution::StorageUnavailable(error.storage_unavailable.expect("typed lineage export refusal")),
                    other => panic!("funded live capture request: {other:?}"),
                }),
            Self::EmbeddedCheckpoint => runtime.durability_authority().native_checkpoint(execution.allocation_policy())
                .map(|checkpoint| checkpoint.bytes().len())
                .map_err(|denial| match denial {
                    crate::durability::data::RelationalNativeCheckpointCaptureDenial::Durability(error) =>
                        Execution::StorageUnavailable(error.storage_unavailable.expect("typed lineage member export refusal")),
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
#[test]
fn residency_failure_matrix_lineage_event_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in EventDoor::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = reopened(execution);
            let directory = fixture
                .runtime
                .durability
                .checkpoint_selection()
                .snapshot()
                .unwrap();
            let page = directory
                .manifest
                .lineage_events
                .iter()
                .find(|page| {
                    page.event_ids
                        .iter()
                        .any(|id| fixture.expected.traversed_event_ids.contains(id))
                })
                .unwrap();
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Member(page.member), cause);
            let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Lineage {
                    first: crate::identity::data::LineageId(page.first_node),
                    last: crate::identity::data::LineageId(page.last_node),
                },
                cause,
            });
            assert_eq!(
                door.read(&fixture, execution),
                Err(expected),
                "{door:?} x {cause:?}"
            );
            fixture.source.failures.lock().unwrap().clear();
            let reads = || {
                fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|member| **member == page.member)
                    .count()
            };
            let before = reads();
            if sticky {
                assert_eq!(door.read(&fixture, execution), Err(expected), "{door:?}");
                assert_eq!(
                    EventDoor::Historical.read(&fixture, execution),
                    Err(expected),
                    "export shares the same damage cell"
                );
                assert_eq!(reads(), before);
            } else {
                assert!(
                    door.read(&fixture, execution).unwrap() > 0,
                    "retry must return genuine history"
                );
                assert!(reads() > before);
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(
                attached_test_medium(&fixture.runtime).offered().len(),
                offered
            );
        }
    }
}
