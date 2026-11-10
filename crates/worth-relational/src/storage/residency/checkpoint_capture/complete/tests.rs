//! Genuine published Struct/history source, prior whole wire and actual custody.
use super::*;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::storage::residency::region_image::{
    RegionHeadImage, RegionHistoryImage, RelationalRegionImage, RelationalRegionImageKind,
};
use crate::storage::residency::PreparedResidentRegion;
use crate::tests::support::*;
use crate::transactions::data::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{AspectValue, ContractValidationInput};

struct Fixture {
    prepared: PreparedResidentRegion,
    schema: Arc<RelationalBranchRootSchemaAuthority>,
    symbols: crate::symbols::data::StringInterner,
    schema_digest: [u8; 32],
}
// The sole Runtime factory is the existing genuine support schema fixture.
fn fixture() -> Fixture {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let make_entity = |name: &str| -> EntityId {
        let fields = std::collections::BTreeMap::from([
            (
                planned_single_field_locator(aspect_key("name"), field_key("name")),
                AspectValue::String(name.into()),
            ),
            (
                planned_single_field_locator(aspect_key("summary"), field_key("title")),
                AspectValue::String(format!("title-{name}").into()),
            ),
            (
                planned_single_field_locator(aspect_key("summary"), field_key("status")),
                AspectValue::String("open".into()),
            ),
        ]);
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new(name).push(MutationIntent::Create(CreateIntent::Entity(
                    EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw(name),
                        fields: AspectFieldPatch::new(fields),
                    },
                ))),
                request.allocation_policy(),
            )
            .unwrap();
        let committed = transaction.commit(&request, &runtime).unwrap();
        let entity = changed_entities(&committed)[0];
        release_test_commit_snapshot(&runtime, &committed);
        entity
    };
    let source = make_entity("original-source");
    let target = make_entity("original-target");
    let later = make_entity("original-later");
    let relation = create_relation(&request, &runtime, source, target, "original-edge");
    for next in [later, target, later] {
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("original-endpoints").push(MutationIntent::Relation(
                    RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                        relation_id: relation,
                        kind_id: KindId(2),
                        source: EntityReference::Existing(source),
                        target: EntityReference::Existing(next),
                    }),
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let committed = transaction.commit(&request, &runtime).unwrap();
        release_test_commit_snapshot(&runtime, &committed);
    }
    let root = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap()
        .root()
        .unwrap();
    let partition = root
        .partition_state(source.partition_id)
        .unwrap()
        .unwrap()
        .clone();
    assert!(
        partition
            .relation_arena
            .metadata_history_at(relation.slot_index())
            .unwrap()
            .len()
            > 2
    );
    let schema = root.retained_schema_authority();
    let schema_digest = crate::durability::data::DurableBranchRootSchemaImage::capture(&schema)
        .unwrap()
        .carrier_digest();
    let symbols = runtime.services.symbols.interner_snapshot();
    let prepared = PreparedResidentRegion::initial(partition, &symbols).unwrap();
    drop(root);
    drop(runtime);
    Fixture {
        prepared,
        schema,
        symbols,
        schema_digest,
    }
}
fn captured(fixture: &Fixture, request: &ExecutionRequest<'_, '_>) -> CapturedCompleteRegion {
    fixture
        .prepared
        .retention
        .cell()
        .capture_complete_for_request(
            request,
            &fixture.schema,
            fixture.schema_digest,
            &fixture.symbols,
        )
        .unwrap()
}
fn bytes(member: &RelationalCheckpointMember) -> &worth_execution::ExecutionImmutableBytes {
    match member {
        RelationalCheckpointMember::New { bytes, .. } => bytes,
        _ => panic!("Complete emits original Full bytes"),
    }
}
#[test]
fn complete_full_reg6_matches_prior_nontrivial_head_and_history_bytes() {
    let fixture = fixture();
    let region = fixture
        .prepared
        .retention
        .capture_region()
        .unwrap()
        .unwrap();
    let old = crate::storage::residency::capture_region_members(
        &region.partition,
        region.commitment.as_ref().unwrap(),
        &fixture.schema,
        fixture.schema_digest,
    )
    .unwrap();
    let host = crate::test_execution_policy::serial_request();
    let emitted = captured(&fixture, &ExecutionRequest::serial(&host));
    assert_eq!(bytes(&emitted.members[0]).as_ref(), old.head.1);
    assert_eq!(bytes(&emitted.members[1]).as_ref(), old.history.1);
    assert_eq!(emitted.leaf.head, old.head.0);
    assert_eq!(emitted.leaf.history, old.history.0);
    assert_eq!(emitted.leaf.kind_authority, old.kind_authority);
    let frame = RelationalRegionImage::decode(
        bytes(&emitted.members[0]),
        emitted.leaf.head,
        emitted.leaf.partition_id,
        fixture.schema_digest,
        RelationalRegionImageKind::Head,
    )
    .unwrap();
    let head: RegionHeadImage = rmp_serde::from_slice(frame.payload).unwrap();
    assert!(head.partition.entity_arena.extra.iter().any(|extra|extra.authoritative_aspect_state.as_ref().is_some_and(|state|
        state.entries().iter().any(|entry|matches!(entry.value(),ContractValidationInput::Struct(value) if value.fields().count()>1)))));
    let (_, frontiers) = head.history_frontiers.into_arenas();
    assert!(frontiers
        .iter()
        .any(|(_, frontier)| frontier.last.is_some() && !frontier.siblings.is_empty()));
    let frame = RelationalRegionImage::decode(
        bytes(&emitted.members[1]),
        emitted.leaf.history,
        emitted.leaf.partition_id,
        fixture.schema_digest,
        RelationalRegionImageKind::History,
    )
    .unwrap();
    let history: RegionHistoryImage = rmp_serde::from_slice(frame.payload).unwrap();
    assert!(history.relations.iter().any(|row| row.earlier.len() > 1));
    assert!(
        history
            .reverse_adjacency_kinds
            .iter()
            .filter(|row| !row.kinds.is_empty())
            .count()
            > 1
    );
    assert!(history.relations.iter().all(|row| row.prefix.is_none()));
    assert_eq!(
        region
            .commitment
            .as_ref()
            .unwrap()
            .digest(region.partition.partition_id),
        emitted.leaf.content_digest
    );
}
#[test]
fn complete_region_ack_busy_and_stop_preserve_original_binding_then_refund() {
    let fixture = fixture();
    let cell = fixture.prepared.retention.cell();
    let budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let first = captured(&fixture, &request);
    let held = cell.checkpoint_bindings.lock().unwrap();
    assert!(matches!(
        first.publication.try_acknowledge_for_request(&request),
        Err(RegionCheckpointAcknowledgementDenial::Busy)
    ));
    assert!(held.is_empty());
    drop(held);
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let stopped = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    assert!(matches!(
        first
            .publication
            .try_acknowledge_for_request(&ExecutionRequest::serial(&stopped)),
        Err(RegionCheckpointAcknowledgementDenial::Stopped(
            RelationalRecoveryStop::Request(_)
        ))
    ));
    assert!(cell.checkpoint_bindings.lock().unwrap().is_empty());
    drop(
        first
            .publication
            .try_acknowledge_for_request(&request)
            .unwrap(),
    );
    assert_eq!(
        cell.checkpoint_bindings
            .lock()
            .unwrap()
            .get(&fixture.schema_digest)
            .unwrap()
            .leaf,
        first.leaf
    );
    let next = captured(&fixture, &request);
    let displaced = next
        .publication
        .try_acknowledge_for_request(&request)
        .unwrap()
        .unwrap();
    assert_eq!(
        displaced
            ._original
            .get(&fixture.schema_digest)
            .unwrap()
            .leaf,
        first.leaf
    );
    drop(first);
    drop(next);
    drop(fixture);
    assert!(
        budget.charged_memory_bytes() > 0,
        "actual displaced original SDK owner still holds its grants"
    );
    drop(displaced);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn complete_region_emitted_byte_alias_retains_only_its_actual_grant() {
    let fixture = fixture();
    let denied_budget = SerialMemoryBudget::new(Some(0));
    let denied = SerialRequest::from_memory(
        denied_budget.clone(),
        CancellationSource::new().token(),
        None,
    );
    assert!(matches!(
        fixture
            .prepared
            .retention
            .cell()
            .capture_complete_for_request(
                &ExecutionRequest::serial(&denied),
                &fixture.schema,
                fixture.schema_digest,
                &fixture.symbols
            ),
        Err(RelationalRecoveryStop::Request(_))
    ));
    assert_eq!(denied_budget.charged_memory_bytes(), 0);
    assert!(fixture
        .prepared
        .retention
        .cell()
        .checkpoint_bindings
        .lock()
        .unwrap()
        .is_empty());
    let budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let emitted = captured(&fixture, &ExecutionRequest::serial(&host));
    let alias = bytes(&emitted.members[0]).clone();
    let payload_bill = alias.charged_payload_bytes().unwrap();
    assert!(payload_bill > 0);
    // The SDK owns its private byte-control layout. Independently construct one
    // real public byte owner; never duplicate that layout in Native's oracle.
    let reference_budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let reference_host = SerialRequest::from_memory(
        reference_budget.clone(),
        CancellationSource::new().token(),
        None,
    );
    let mut reference = worth_execution::ExecutionRequestArrayBuilder::allocate(
        ExecutionRequest::serial(&reference_host),
        alias.len(),
    )
    .unwrap();
    reference.extend_from_slice(alias.bytes()).unwrap();
    let reference = reference.seal_immutable_bytes().unwrap();
    let bill = reference_budget.charged_memory_bytes();
    assert_eq!(reference.charged_payload_bytes(), Some(payload_bill));
    assert!(
        bill > payload_bill,
        "the actual independent SDK control is funded"
    );
    drop(reference);
    assert_eq!(reference_budget.charged_memory_bytes(), 0);
    assert!(budget.charged_memory_bytes() > bill);
    drop(emitted);
    drop(fixture);
    assert_eq!(budget.charged_memory_bytes(), bill);
    assert!(alias.starts_with(b"WORTH-REL-REG6\n"));
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
