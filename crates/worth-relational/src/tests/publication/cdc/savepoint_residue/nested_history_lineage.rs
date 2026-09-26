use crate::facade::lineage::LineageDecisionKind;
use crate::tests::support::*;

#[test]
fn nested_savepoint_abandoned_aspect_work_leaves_zero_patch_cdc_history_and_lineage_residue() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&runtime, "anchor");
    let anchor = changed_entities(&created)[0];
    let target = create_entity(&runtime, "target");
    let start_lineage = runtime
        .lineage_access()
        .for_record(anchor)
        .unwrap()
        .lineage_id;
    let checkpoint = checkpoint_for_schema_version(
        runtime.publication().latest_patch().unwrap().position,
        SchemaVersionId(1),
    );

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    let savepoint_a = txn.create_savepoint().unwrap();
    txn.push_batch(batch_create("surviving-a"))
        .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("surviving-a-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: anchor,
                fields: single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    field_key("name"),
                    "surviving-a-anchor",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");

    let savepoint_b = txn.create_savepoint().unwrap();
    txn.push_batch(batch_create("abandoned-entity"))
        .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("abandoned-relation").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("abandoned-r"),
                source: crate::transactions::data::EntityReference::Existing(anchor),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("abandoned-replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: anchor,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("abandoned-replacement"),
                    fields: single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        field_key("name"),
                        "abandoned-replacement",
                    ),
                },
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let rollback_b = txn.rollback_to_savepoint(savepoint_b).unwrap();

    txn.push_batch(batch_create("surviving-b"))
        .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("surviving-b-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: anchor,
                fields: single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    field_key("name"),
                    "surviving-b-anchor",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let rollback_a = txn.rollback_to_savepoint(savepoint_a).unwrap();

    txn.push_batch(batch_create("surviving-final"))
        .expect("test staging stays within configured resource budgets");
    txn.push_batch(
        WorkerIntentBatch::new("surviving-final-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: anchor,
                fields: single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    field_key("name"),
                    "surviving-final-anchor",
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.push_batch(WorkerIntentBatch::new("surviving-final-relation").push(
        MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("surviving-r"),
                source: crate::transactions::data::EntityReference::Existing(anchor),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        )),
    ))
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime).unwrap();

    assert!(rollback_b.has_effects());
    assert!(rollback_a.has_effects());
    let _ = assert_patch_truth_invariants(&outcome);
    assert_patch_omits_detail(&outcome, "abandoned");

    assert_subscriber_stream_omits_detail(&runtime, checkpoint, "abandoned");

    let direct_history =
        runtime
            .history()
            .entity_aspect_history(&BranchId("main".to_string()), anchor, None);
    let direct_traced = runtime.history().entity_aspect_history_with_trace(
        &BranchId("main".to_string()),
        anchor,
        None,
    );
    assert_eq!(direct_history.len(), 2);
    assert_eq!(direct_traced.aspect_history_digest().entry_count, 2);
    assert_direct_history_origin_invariants(&direct_history, RecordRef::Entity(anchor));

    let lineage_traced = runtime.lineage_access().entity_aspect_history_with_trace(
        crate::facade::lineage::HistoricalResolutionRequest {
            branch_id: BranchId("main".to_string()),
            lineage_id: start_lineage,
            boundedness_basis:
                crate::facade::lineage::HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
        },
        None,
    );
    let lineage_history = lineage_traced
        .history
        .as_ref()
        .expect("lineage aspect history");
    assert_eq!(lineage_history.traversed_event_ids.len(), 0);
    assert_eq!(lineage_history.entries.len(), 2);
    assert_lineage_history_origin_invariants(&lineage_history.entries, start_lineage);
    assert_eq!(
        lineage_traced
            .lineage_aspect_resolution_digest()
            .traversed_lineage_events,
        0
    );

    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .unwrap();
    let entity_names = read
        .entities()
        .iter()
        .filter_map(read_entity_name)
        .collect::<Vec<_>>();

    assert!(entity_names.contains(&"target".into()));
    assert!(entity_names.contains(&"surviving-final".into()));
    assert!(entity_names.contains(&"surviving-final-anchor".into()));
    assert!(!entity_names.iter().any(|name| name.contains("abandoned")));
    assert_eq!(read.relations().len(), 1);
    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(outcome.commit.commit_id)
        .unwrap();
    assert!(!envelope
        .lineage_decision_log()
        .iter()
        .any(|decision| decision.kind == LineageDecisionKind::ReplaceAccepted));
}
