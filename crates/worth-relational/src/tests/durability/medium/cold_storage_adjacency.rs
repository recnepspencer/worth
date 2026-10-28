//! Every public whole-adjacency read admits the cold mirror and keeps warm answers equal.
use super::CurrentFixture;
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{EntityId, RelationId};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

#[derive(Clone, Copy, Debug)]
enum Door {
    Outgoing,
    Incoming,
    All,
}
impl Door {
    fn read(
        self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<Vec<RelationId>, RelationalExecutionDenialCause> {
        let access = runtime.storage_access();
        let version = runtime.current_version_id();
        match self {
            Self::Outgoing => access.outgoing_relations_for_entity(execution, entity, version),
            Self::Incoming => access.incoming_relations_for_entity(execution, entity, version),
            Self::All => access.all_relations_for_entity(execution, entity, version),
        }
    }
}
#[test]
fn residency_storage_adjacency_doors_demand_before_reading_and_preserve_cold_warm_answers() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    for door in [Door::Outgoing, Door::Incoming, Door::All] {
        let fixture = CurrentFixture::reopened(&execution);
        let entity = if matches!(door, Door::Incoming) {
            fixture.target
        } else {
            fixture.entity
        };
        assert_eq!(
            fixture
                .source
                .reads
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert_eq!(
            door.read(&fixture.runtime, &execution, entity).unwrap(),
            vec![fixture.relation],
            "{door:?}"
        );
        let reads = fixture
            .source
            .reads
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(reads > 0, "the cold public door must demand its mirror");
        assert_eq!(
            door.read(&fixture.runtime, &execution, entity).unwrap(),
            vec![fixture.relation]
        );
        assert_eq!(
            fixture
                .source
                .reads
                .load(std::sync::atomic::Ordering::Relaxed),
            reads
        );
    }
}
