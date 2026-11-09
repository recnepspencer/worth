//! Shipping node consumers share the actual page refusal contract.
use super::lineage_node_source::NodeFixture;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    change_source::RelationalObservationReadDenial as Observation, durability::*,
    history::BranchId, lineage::RelationalLineageDiscoveryDenial as Discovery,
    transactions::RelationalExecutionDenialCause as Execution,
};
#[derive(Clone, Copy, Debug)]
pub(super) enum NodeDoor {
    MemoryCheckpoint,
    EmbeddedCheckpoint,
    Point,
    RecordHistory,
    AspectHistory,
    AspectHistoryTrace,
    ObservationVisible,
}
impl NodeDoor {
    pub const ALL: [Self; 7] = [
        Self::MemoryCheckpoint,
        Self::EmbeddedCheckpoint,
        Self::Point,
        Self::RecordHistory,
        Self::AspectHistory,
        Self::AspectHistoryTrace,
        Self::ObservationVisible,
    ];
    pub fn read(
        self,
        fixture: &NodeFixture,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Execution> {
        let runtime = &fixture.runtime;
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        match self {
            Self::Point => runtime
                .lineage_node_at_observation(
                    request,
                    &basis.observation(),
                    fixture.expected.entity_id(),
                )
                .map(|node| {
                    assert_eq!(
                        node.as_ref().map(|read| read.node()),
                        Some(&fixture.expected)
                    )
                })
                .map_err(observation),
            Self::RecordHistory => runtime
                .discover_authored_record_history(
                    request,
                    &runtime.main_branch_identity(),
                    fixture.expected.entity_id(),
                )
                .map(|history| {
                    assert!(history
                        .expect("the producer has a lineage")
                        .resolved
                        .contains(&fixture.expected.lineage_id()))
                })
                .map_err(discovery),
            Self::ObservationVisible => runtime
                .visible_entities_for_lineages_at_observation(
                    request,
                    &basis.observation(),
                    &[fixture.expected.lineage_id()],
                )
                .map(|entities| assert!(entities.contains(&fixture.expected.entity_id())))
                .map_err(observation),
            Self::AspectHistory => runtime
                .history()
                .lineage_entity_aspect_history(
                    request,
                    &BranchId("main".into()),
                    fixture.expected.lineage_id(),
                    None,
                )
                .map(|history| {
                    let history = history.expect("the producer has aspect history");
                    assert!(!history.entries.is_empty());
                    assert!(history
                        .resolved_lineage_chain
                        .contains(&fixture.expected.lineage_id()));
                }),
            Self::AspectHistoryTrace => runtime
                .history()
                .lineage_entity_aspect_history_with_trace(
                    request,
                    &BranchId("main".into()),
                    fixture.expected.lineage_id(),
                    None,
                )
                .map(|result| {
                    let history = result
                        .history
                        .expect("the producer has traced aspect history");
                    assert!(!history.entries.is_empty());
                    assert!(history
                        .resolved_lineage_chain
                        .contains(&fixture.expected.lineage_id()));
                    assert!(result.trace.returned_entries > 0);
                }),
            Self::MemoryCheckpoint => runtime
                .durability_authority()
                .checkpoint(request)
                .map(|image| assert!(!image.partition_images.is_empty()))
                .map_err(memory_stop),
            Self::EmbeddedCheckpoint => runtime
                .durability_authority()
                .native_checkpoint(request, request.allocation_policy())
                .map(|image| assert!(!image.bytes().is_empty()))
                .map_err(native_stop),
        }
    }
}
fn observation(denial: Observation) -> Execution {
    match denial {
        Observation::Execution(cause) => cause,
        Observation::ForeignObservation => panic!("fixture owns its observation"),
    }
}
fn discovery(denial: Discovery) -> Execution {
    match denial {
        Discovery::Execution(cause) => cause,
        Discovery::Branch(cause) => panic!("fixture owns its branch: {cause:?}"),
        Discovery::PublicationInFlight => panic!("fixture has no writer"),
        Discovery::PerformedPublicationRequiresSettlement(commit) => {
            panic!("fixture settled {commit:?}")
        }
    }
}
fn memory_stop(stop: RelationalRecoveryStop) -> Execution {
    match stop {
        RelationalRecoveryStop::Durability(error) => {
            Execution::StorageUnavailable(error.storage_unavailable.expect("typed image refusal"))
        }
        RelationalRecoveryStop::AdmissionImage(cause) => panic!("original image: {cause:?}"),
        RelationalRecoveryStop::Request(cause) => panic!("funded request: {cause:?}"),
        RelationalRecoveryStop::Allocation(cause) => panic!("funded allocation: {cause:?}"),
        RelationalRecoveryStop::Content(cause) => panic!("original content: {cause:?}"),
    }
}
fn native_stop(stop: RelationalNativeCheckpointCaptureDenial) -> Execution {
    match stop {
        RelationalNativeCheckpointCaptureDenial::Durability(error) => {
            Execution::StorageUnavailable(error.storage_unavailable.expect("typed export refusal"))
        }
        RelationalNativeCheckpointCaptureDenial::AdmissionImage(cause) => {
            panic!("original image: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::Request(cause) => {
            panic!("funded request: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::Allocation(cause) => {
            panic!("funded allocation: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::Content(cause) => {
            panic!("original content: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause) => {
            panic!("original frame: {cause:?}")
        }
        RelationalNativeCheckpointCaptureDenial::OwnerUnavailable => panic!("live owner"),
        RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable => {
            panic!("standalone original export")
        }
        RelationalNativeCheckpointCaptureDenial::RetentionCapacityExhausted => {
            panic!("fixture has retention headroom")
        }
        RelationalNativeCheckpointCaptureDenial::RetentionIdentityExhausted => {
            panic!("fixture has identity headroom")
        }
    }
}
