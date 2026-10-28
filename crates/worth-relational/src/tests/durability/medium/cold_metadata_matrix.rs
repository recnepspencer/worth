//! Current metadata must use admitted Head without promoting history.
use super::*;
#[test]
fn head_resident_metadata_refuses_to_open_unavailable_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    CurrentDoor::Entity
        .run(
            execution,
            &fixture.runtime,
            fixture.entity,
            fixture.relation,
        )
        .unwrap();
    fixture.source.failures.lock().unwrap().extend(
        fixture
            .history
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::SourceUnavailable)),
    );
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let view = fixture
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    assert_eq!(
        view.read_exact_entity_state(execution, fixture.entity, |metadata, _| metadata.entity_id)
            .unwrap(),
        Some(fixture.entity)
    );
    assert_eq!(
        view.read_exact_relation_metadata(execution, fixture.relation)
            .unwrap()
            .unwrap()
            .relation_id,
        fixture.relation
    );
    assert!(view
        .read_entity_aspect_version(execution, fixture.entity, &aspect_key("name"))
        .unwrap()
        .unwrap()
        .is_some());
    assert!(view
        .read_exact_adjacency_structural_revision(
            execution,
            fixture.entity,
            KindId(2),
            crate::runtime::RelationalAdjacencyDirection::Outgoing
        )
        .unwrap()
        .is_some());
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .iter()
        .any(|member| fixture.history.contains(member)));
}
