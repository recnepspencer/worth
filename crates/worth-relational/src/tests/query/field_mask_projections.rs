use crate::facade::identity::EntityId;
use crate::facade::runtime::{
    EntityProjectionRecord, EntityRecordProjection, ProjectionAspectScope,
    RelationProjectionRecord, RelationRecordProjection,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    aspects, AspectIdentity, AspectValue, FieldKey, InternedString, ScalarAspectType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct SummaryTitleFieldProjection {
    entity_id: EntityId,
    title: String,
    status_was_visible: bool,
    whole_struct_was_visible: bool,
}

impl EntityRecordProjection for SummaryTitleFieldProjection {
    const KIND: KindId = KindId(1);

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::fields(
            AspectKey::new("summary").unwrap(),
            [FieldKey::new("title").unwrap()],
        )
    }

    fn from_record(record: EntityProjectionRecord<'_>) -> Option<Self> {
        let summary = AspectKey::new("summary").unwrap();
        let title_field = FieldKey::new("title").unwrap();
        let status_field = FieldKey::new("status").unwrap();
        let AspectValue::String(title) = record.aspect_field_value(&summary, &title_field)? else {
            return None;
        };
        Some(Self {
            entity_id: record.entity_id(),
            title: raw_interned_string(title)?.to_string(),
            status_was_visible: record.aspect_field_value(&summary, &status_field).is_some(),
            whole_struct_was_visible: record.struct_aspect_value(&summary).is_some(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SummaryWholeAspectProjection {
    title: String,
}

impl EntityRecordProjection for SummaryWholeAspectProjection {
    const KIND: KindId = KindId(1);

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::whole_aspects([AspectKey::new("summary").unwrap()])
    }

    fn from_record(record: EntityProjectionRecord<'_>) -> Option<Self> {
        let summary = record.struct_aspect_value(&AspectKey::new("summary").unwrap())?;
        let AspectValue::String(title) = summary.get(&FieldKey::new("title").unwrap())? else {
            return None;
        };
        Some(Self {
            title: raw_interned_string(title)?.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NameFieldMaskProjection;

impl EntityRecordProjection for NameFieldMaskProjection {
    const KIND: KindId = KindId(1);

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::fields(
            AspectKey::new("name").unwrap(),
            [FieldKey::new("name").unwrap()],
        )
    }

    fn from_record(_record: EntityProjectionRecord<'_>) -> Option<Self> {
        Some(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RelationSummaryTitleFieldProjection {
    title: String,
    status_was_visible: bool,
    whole_struct_was_visible: bool,
}

impl RelationRecordProjection for RelationSummaryTitleFieldProjection {
    const KIND: KindId = KindId(2);

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::fields(
            AspectKey::new("relation.summary").unwrap(),
            [FieldKey::new("title").unwrap()],
        )
    }

    fn from_record(record: RelationProjectionRecord<'_>) -> Option<Self> {
        let summary = AspectKey::new("relation.summary").unwrap();
        let title_field = FieldKey::new("title").unwrap();
        let status_field = FieldKey::new("status").unwrap();
        let AspectValue::String(title) = record.aspect_field_value(&summary, &title_field)? else {
            return None;
        };
        Some(Self {
            title: raw_interned_string(title)?.to_string(),
            status_was_visible: record.aspect_field_value(&summary, &status_field).is_some(),
            whole_struct_was_visible: record.struct_aspect_value(&summary).is_some(),
        })
    }
}

#[test]
fn entity_field_mask_projection_reads_only_declared_struct_field() {
    let runtime = entity_summary_projection_runtime();
    let entity_id = create_entity_with_summary_fields(
        &runtime,
        "field-mask-entity",
        "visible-title",
        "hidden-status",
    );

    let projected = runtime
        .read_truth()
        .project_historical_version(runtime.current_version_id())
        .entities::<SummaryTitleFieldProjection>();

    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].entity_id, entity_id);
    assert_eq!(projected[0].title, "visible-title");
    assert!(!projected[0].status_was_visible);
    assert!(!projected[0].whole_struct_was_visible);
}

#[test]
fn whole_aspect_projection_still_reads_full_struct_aspect() {
    let runtime = entity_summary_projection_runtime();
    create_entity_with_summary_fields(
        &runtime,
        "whole-summary-entity",
        "whole-title",
        "whole-status",
    );

    let projected = runtime
        .read_truth()
        .project_historical_version(runtime.current_version_id())
        .entities::<SummaryWholeAspectProjection>();

    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].title, "whole-title");
}

#[test]
#[should_panic(expected = "projection mask rejected by aspect contract")]
fn scalar_aspect_rejects_field_mask_projection_at_use_boundary() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity(&runtime, "scalar-field-mask");

    let _ = runtime
        .read_truth()
        .project_historical_version(runtime.current_version_id())
        .entities::<NameFieldMaskProjection>();
}

#[test]
fn relation_field_mask_projection_reads_only_declared_struct_field() {
    let runtime = relation_summary_projection_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    create_relation_with_summary_fields(
        &runtime,
        source,
        target,
        "field-mask-relation",
        "relation-title",
        "relation-status",
    );

    let projected = runtime
        .read_truth()
        .project_historical_version(runtime.current_version_id())
        .relations::<RelationSummaryTitleFieldProjection>();

    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].title, "relation-title");
    assert!(!projected[0].status_was_visible);
    assert!(!projected[0].whole_struct_was_visible);
}

fn raw_interned_string(value: &InternedString) -> Option<&str> {
    match value {
        InternedString::Raw(value) => Some(value.as_str()),
        InternedString::Symbol(_) => None,
    }
}

fn entity_summary_projection_runtime() -> RelationalRuntime {
    AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime()
}

fn relation_summary_projection_runtime() -> RelationalRuntime {
    AspectSchemaFixture {
        entity_aspects: vec![entity_field_aspect(aspect_key("name"), field_key("name"))],
        relation_aspects: vec![
            relation_source_aspect(),
            relation_target_aspect(),
            relation_summary_struct_aspect(
                aspect_key("relation.summary"),
                field_key("relation_summary"),
            ),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime()
}

fn relation_summary_struct_aspect(
    aspect_key: AspectKey,
    field: FieldKey,
) -> DeclaredAspectContractBinding {
    let shape = aspects()
        .struct_fields()
        .required("title", ScalarAspectType::String)
        .optional("status", ScalarAspectType::String)
        .finish()
        .expect("valid relation summary struct aspect shape");
    DeclaredAspectContractBinding {
        binding: AspectBinding::RelationField { field },
        contract: aspects()
            .contract()
            .for_key(aspect_key.clone())
            .identified_by(AspectIdentity(
                9000 + test_projection_contract_identity(&aspect_key),
            ))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(shape),
    }
}

fn create_entity_with_summary_fields(
    runtime: &RelationalRuntime,
    client_key: &str,
    summary_title: &str,
    summary_status: &str,
) -> EntityId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("batch-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                fields: AspectFieldPatch::new(std::collections::BTreeMap::from([
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("name"),
                            field_key("name"),
                        ),
                        string_aspect_value(client_key),
                    ),
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("summary"),
                            field_key("title"),
                        ),
                        string_aspect_value(summary_title),
                    ),
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("summary"),
                            field_key("status"),
                        ),
                        string_aspect_value(summary_status),
                    ),
                ])),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    changed_entities(
        &txn.commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap(),
    )[0]
}

fn create_relation_with_summary_fields(
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    client_key: &str,
    summary_title: &str,
    summary_status: &str,
) {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("relation-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: AspectFieldPatch::new(std::collections::BTreeMap::from([
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("relation.summary"),
                            field_key("title"),
                        ),
                        string_aspect_value(summary_title),
                    ),
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("relation.summary"),
                            field_key("status"),
                        ),
                        string_aspect_value(summary_status),
                    ),
                ])),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
}

fn test_projection_contract_identity(aspect_key: &AspectKey) -> u64 {
    let mut hash = 14695981039346656037_u64;
    for byte in aspect_key.as_str().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211_u64);
    }
    hash
}
