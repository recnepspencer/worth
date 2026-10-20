//! Issued rows, dependencies, revisions and the same bill at every posture.
use super::*;
use crate::identity::data::VersionId;
use crate::storage::data::{RecordLifecycleState, RelationalFieldPresence};
use crate::visibility::materialization::read_records::{
    entity_query_locus_value, relation_query_locus_value,
};
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

mod bounded_index;
mod cases;
mod fixture;
mod refusals;

#[test]
fn runtime_read_law_matrix_keeps_rows_dependencies_revisions_and_work() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    for indexed in [false, true] {
        let world = fixture::World::new(&setup, indexed);
        for case in cases::table(&world) {
            let mut bill = None;
            for (posture, width) in [
                (None, 1),
                (Some(ExecutionPosture::Serial), 1),
                (Some(ExecutionPosture::Automatic), 1),
                (Some(ExecutionPosture::Automatic), 4),
            ] {
                let lease = posture.map(|posture| {
                    test_execution_authority()
                        .request_lease(LeaseRequest {
                            policy: ExecutionRequestPolicy::new(
                                posture,
                                DeterminismContract::CanonicalBitwise,
                                ExecutionBudget::new(
                                    std::num::NonZeroUsize::new(width).unwrap(),
                                    None,
                                    Some(100_000),
                                ),
                            ),
                            deadline: None,
                            cancellation: CancellationToken::new(),
                        })
                        .unwrap()
                });
                let request = lease
                    .as_ref()
                    .map_or(ExecutionRequest::serial(&host), ExecutionRequest::leased);
                let read = world
                    .runtime
                    .read_truth()
                    .execute_query_plan(world.plan(&case), &request)
                    .unwrap()
                    .unwrap();
                let result = &read.outcome.result;
                assert_eq!(
                    result
                        .entities
                        .iter()
                        .map(|row| row.entity_id)
                        .collect::<Vec<_>>(),
                    case.entities
                        .iter()
                        .map(|&i| world.entities[i])
                        .collect::<Vec<_>>(),
                    "{} {posture:?}/{width}, indexed={indexed}",
                    case.name
                );
                assert_eq!(
                    result
                        .relations
                        .iter()
                        .map(|row| row.relation_id)
                        .collect::<Vec<_>>(),
                    case.relations
                        .iter()
                        .map(|&i| world.relations[i])
                        .collect::<Vec<_>>(),
                    "{}",
                    case.name
                );
                // Five seed commits establish this exact dependency basis.
                assert_eq!(read.outcome.plan.packet.context_id.version_id, VersionId(5));
                assert_eq!(read.outcome.plan.snapshot, world.snapshot);
                assert_eq!(
                    read.outcome.plan.packet.context_id.schema_version,
                    SchemaVersionId(1)
                );
                let projection = world
                    .runtime
                    .read_truth()
                    .project_snapshot(&world.snapshot)
                    .unwrap();
                for (row, &i) in result.entities.iter().zip(case.entities) {
                    assert_eq!(row.created_at_version, VersionId((i + 1) as u64));
                    assert_eq!(row.retired_at_version, None);
                    assert_eq!(row.lifecycle, RecordLifecycleState::Live);
                    assert_eq!(row.kind.kind_id, KindId(1));
                    assert_eq!(
                        entity_query_locus_value(row, &world.name),
                        Some(&string_aspect_value(["a", "b", "c"][i]))
                    );
                    let revision = projection
                        .entity_field_revision(&request, row.entity_id, &world.name)
                        .unwrap()
                        .unwrap();
                    assert_eq!(revision.version(), VersionId((i + 1) as u64));
                    assert_eq!(revision.presence(), RelationalFieldPresence::Present);
                }
                for (row, &i) in result.relations.iter().zip(case.relations) {
                    assert_eq!(row.created_at_version, VersionId((i + 4) as u64));
                    assert_eq!(row.retired_at_version, None);
                    assert_eq!(row.lifecycle, RecordLifecycleState::Live);
                    assert_eq!(row.kind.kind_id, KindId(2));
                    assert_eq!(
                        (row.source, row.target),
                        (world.entities[i], world.entities[i + 1])
                    );
                    assert_eq!(
                        relation_query_locus_value(row, &world.label),
                        Some(&string_aspect_value(["ab", "bc"][i]))
                    );
                    let revision = projection
                        .relation_field_revision(&request, row.relation_id, &world.label)
                        .unwrap()
                        .unwrap();
                    assert_eq!(revision.version(), VersionId((i + 4) as u64));
                    assert_eq!(revision.presence(), RelationalFieldPresence::Present);
                }
                let report = read.report.unwrap();
                let actual = (report.charged_work(), report.charged_span());
                if let Some(expected) = bill {
                    assert_eq!(
                        actual, expected,
                        "{} {posture:?}/{width}, indexed={indexed}",
                        case.name
                    );
                } else {
                    bill = Some(actual);
                }
            }
        }
    }
}

mod remaining_doors;

mod remaining_door_stops;
