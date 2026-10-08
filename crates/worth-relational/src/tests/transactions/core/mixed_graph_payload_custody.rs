//! Real mixed transactions preserve existing aspect custody across sparse-plan limits.
use crate::facade::symbols::ClientKey;
use crate::facade::transactions::{CreatedEntityRef, EntityReference, EntitySpec, RelationSpec};
use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};

#[test]
fn mixed_graph_creation_preserves_existing_struct_patch_at_eight_and_nine_partitions() {
    for partitions in [8, 9] {
        run_mixed_graph(partitions);
    }
}

fn run_mixed_graph(partitions: u32) {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut initial = test_owner_begin_transaction_for_main(&runtime);
    let mut batch = WorkerIntentBatch::new("actual-existing-partitions");
    let references = (0..partitions)
        .map(|partition| created(partition, &format!("existing-{partition}")))
        .collect::<Vec<_>>();
    for reference in &references {
        batch = batch.push(create(reference));
    }
    initial
        .push_batch(batch, AllocationPolicy::SystemAllocation)
        .unwrap();
    let initial_commit = initial
        .commit(&runtime, AllocationPolicy::SystemAllocation)
        .expect("the initial records have genuinely committed aspect custody");
    let existing = references
        .iter()
        .map(|reference| initial_commit.created_entity(reference).unwrap())
        .collect::<Vec<_>>();
    let owner = existing[0];
    let mut batch = WorkerIntentBatch::new("atomic-mixed-cohort");
    let mut cohort = Vec::new();
    for (index, body) in existing.iter().enumerate().skip(1) {
        let member = created(0, &format!("member-{index}"));
        let occurrence = created(0, &format!("occurrence-{index}"));
        batch = batch.push(create(&member)).push(create(&occurrence));
        for (role, source, target) in [
            (
                "owner",
                EntityReference::Existing(owner),
                EntityReference::Created(member.clone()),
            ),
            (
                "member",
                EntityReference::Created(member.clone()),
                EntityReference::Created(occurrence.clone()),
            ),
            (
                "geometry",
                EntityReference::Created(occurrence.clone()),
                EntityReference::Existing(*body),
            ),
        ] {
            batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw(format!("{role}-{index}")),
                    source,
                    target,
                    fields: AspectFieldPatch::default(),
                },
            )));
        }
        cohort.push((member, occurrence, *body));
    }
    batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: owner,
            fields: single_string_aspect_field_patch(
                aspect_key("summary"),
                field_key("title"),
                "published",
            ),
        },
    )));
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch, AllocationPolicy::SystemAllocation)
        .unwrap();
    let proposal = transaction
        .validate(&runtime, AllocationPolicy::SystemAllocation)
        .expect("relation eligibility cannot remove required entity payloads");
    let before = runtime
        .read_truth()
        .read_snapshot(&initial_commit.snapshot)
        .unwrap();
    assert_summary(&before.get_entity(owner).unwrap(), "pending");
    let candidate = runtime.prepare_validated_proposal(proposal).unwrap();
    let committed = runtime.publish_prepared_candidate(candidate).unwrap();
    let read = runtime
        .read_truth()
        .read_snapshot(&committed.snapshot)
        .unwrap();
    let owner_record = read
        .get_entity(owner)
        .expect("original owner identity persists");
    assert_summary(&owner_record, "published");
    assert_eq!(read_entity_name(&owner_record), Some("existing-0".into()));
    for (member, occurrence, body) in cohort {
        let member_id = committed.created_entity(&member).unwrap();
        let occurrence_id = committed.created_entity(&occurrence).unwrap();
        assert_ne!(member_id, occurrence_id);
        assert!(read.get_entity(member_id).is_some());
        assert!(read.get_entity(occurrence_id).is_some());
        assert_summary(
            &read
                .get_entity(body)
                .expect("unchanged geometry identity persists"),
            "pending",
        );
        let relations = changed_relations(&committed)
            .into_iter()
            .map(|id| read.get_relation(id).unwrap())
            .collect::<Vec<_>>();
        for (source, target) in [
            (owner, member_id),
            (member_id, occurrence_id),
            (occurrence_id, body),
        ] {
            assert_eq!(
                relations
                    .iter()
                    .filter(|record| record.source == source && record.target == target)
                    .count(),
                1
            );
        }
    }
    assert_eq!(
        changed_relations(&committed).len(),
        3 * (partitions - 1) as usize
    );
}

fn created(partition: u32, key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId(partition),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
    }
}
fn create(reference: &CreatedEntityRef) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: reference.partition_id,
        kind_id: reference.kind_id,
        client_key: reference.client_key.clone(),
        fields: string_aspect_field_patch([
            (
                aspect_key("name"),
                field_key("name"),
                reference.client_key.as_raw_str().unwrap(),
            ),
            (aspect_key("summary"), field_key("title"), "pending"),
            (aspect_key("summary"), field_key("status"), "retained"),
        ]),
    }))
}
fn assert_summary(record: &crate::facade::runtime::EntityReadRecord, title: &str) {
    let aspect = record
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(&aspect_key("summary"))
        .unwrap();
    let ContractValidatedAspectValueView::Struct(value) = aspect.view() else {
        panic!("summary remains a struct");
    };
    assert_eq!(
        value.get(&field_key("title")),
        Some(&AspectValue::String(title.into()))
    );
    assert_eq!(
        value.get(&field_key("status")),
        Some(&AspectValue::String("retained".into()))
    );
}
