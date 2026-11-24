use super::{KindScanVisibility, ProjectionRecordReader};
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::{
    EntityKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::storage::data::EntityReadRecord;
use crate::storage::overlay::small_relation_partition::relation_partition;
use crate::storage::overlay::PartitionState;
use std::collections::BTreeMap;

fn resident_row() -> (
    BTreeMap<PartitionId, PartitionState>,
    RelationalSchemaRegistry,
    crate::identity::data::EntityId,
) {
    let id = PartitionId::main();
    let (partition, entities, _) = relation_partition(id, 1, &[]);
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "native entity".to_owned(),
            schema_id: SchemaId("row-quote".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    (BTreeMap::from([(id, partition)]), registry, entities[0])
}
use crate::visibility::materialization::read_records::{read_footprint, read_work};
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn finite_native_row_refuses_before_cloning_and_accepts_its_owned_quote() {
    let (state, registry, entity) = resident_row();
    let version = VersionId(1);
    let instrumentation = RuntimeInstrumentation::default();
    let reader = ProjectionRecordReader::new(version, &instrumentation);
    let heap = read_work::record_quote::entity_for_id(
        &state,
        &registry,
        entity,
        KindScanVisibility::for_selected_version(version, version),
    )
    .unwrap();
    assert!(heap > 0, "the native schema names have an owned clone");
    let accounting = SerialMemoryBudget::new(None);
    let probe = SerialRequest::from_memory(accounting.clone(), CancellationToken::new(), None);
    let mut opening = None;
    read_work::metered_with_memory(
        &ExecutionRequest::serial(&probe),
        |_context, _memory| {
            opening = Some(accounting.charged_memory_bytes());
            Ok::<Option<EntityReadRecord>, Cause>(None)
        },
        |_| 0,
    )
    .unwrap()
    .unwrap();
    let opening = opening.expect("the identical result carrier entered its real framework scope");
    for funded in [false, true] {
        let limit = opening.checked_add(heap).unwrap() - u64::from(!funded);
        let memory = SerialMemoryBudget::new(Some(limit));
        let request = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
        let rows = crate::visibility::materialization::read_records::materialization::row_reads_for_test::RowReadsForTest::install_on_this_thread();
        let result = reader.read_live_entity_at_version(
            &ExecutionRequest::serial(&request),
            &state,
            &registry,
            entity,
            version,
        );
        if funded {
            let record = result
                .unwrap()
                .expect("the real native identity remains visible");
            assert_eq!(record.entity_id, entity);
            assert_eq!(read_footprint::entity(&record), heap);
            assert_eq!(rows.rows(), 1);
        } else {
            assert!(matches!(result, Err(Cause::PolicyMemoryExhausted { .. })));
            assert_eq!(rows.rows(), 0, "refusal precedes the original native clone");
        }
        assert_eq!(
            memory.charged_memory_bytes(),
            0,
            "the original scoped reservation settles"
        );
    }
}

#[test]
fn native_clone_settles_source_spare_payload_on_the_original_ticket() {
    let host = crate::execution::test_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (state, registry, entity) = resident_row();
    let version = VersionId(1);
    let instrumentation = RuntimeInstrumentation::default();
    let reader = ProjectionRecordReader::new(version, &instrumentation);
    let mut source = reader
        .read_live_entity_at_version(&execution, &state, &registry, entity, version)
        .unwrap()
        .unwrap();
    source.kind.kind_name.reserve(4096);
    let quoted = read_footprint::entity(&source);
    let answer = read_work::metered_with_memory(
        &execution,
        |_context, memory| {
            read_work::record_memory::claim(memory, quoted)?;
            assert_eq!(memory.bytes(), quoted);
            let record = source.clone();
            let actual = read_footprint::entity(&record);
            assert!(
                actual < quoted,
                "the real native source retains spare string capacity"
            );
            read_work::record_memory::settle_clone(memory, quoted, actual)?;
            assert_eq!(
                memory.bytes(),
                actual,
                "settlement shrinks the original result loan"
            );
            Ok::<_, Cause>(record)
        },
        |answer| answer.as_ref().map_or(0, read_footprint::entity),
    )
    .unwrap()
    .unwrap();
    assert_eq!(answer.entity_id, entity);
    assert!(source.kind.kind_name.capacity() > answer.kind.kind_name.capacity());
}
