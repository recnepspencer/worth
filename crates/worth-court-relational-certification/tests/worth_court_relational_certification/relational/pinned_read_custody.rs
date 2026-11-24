//! Exact pins retain their original traversal frontier and observation state.
use super::pinned_read_publication::snapshot_for_main;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView, InternedString};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, RelationId, VersionId},
    query::*,
    runtime::{RelationalPinnedReadAdmissionDenial, RelationalRetainedCommitSnapshotDenialKind},
    transactions::{
        CreateIntent, EntityReference, EntitySpec, MutationIntent, RecordRef, RelationSpec,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key,
        publications::{begin_main_transaction, changed_entities, release_test_commit_snapshot},
    },
    durable_publication::{create_entity_outcome, field_patch, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn pinned_traversal_keeps_its_frontier_across_a_commit() {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let publish = |label: &str, intent| {
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new(label).push(MutationIntent::Create(intent)),
                request.allocation_policy(),
            )
            .unwrap();
        let result = transaction.commit(&request, &runtime).unwrap();
        release_test_commit_snapshot(&runtime, &result);
        result
    };
    let create_entity = |name: &str| -> EntityId {
        changed_entities(&publish(
            name,
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId(7),
                kind_id: KindId(1),
                client_key: name.into(),
                fields: field_patch("name", "name", name),
            }),
        ))[0]
    };
    let create_relation = |name: &str, source, target| -> RelationId {
        let result = publish(
            name,
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId(7),
                kind_id: KindId(2),
                client_key: name.into(),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: field_patch("label", "label", name),
            }),
        );
        let [RecordRef::Relation(relation)] = result.changed_records.as_slice() else {
            panic!("fixture creates one relation");
        };
        *relation
    };
    let a = create_entity("a");
    let b = create_entity("b");
    let ab = create_relation("ab", a, b);
    let snapshot = snapshot_for_main(&runtime);
    let pinned = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[], &request)
        .unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "held-frontier".into(),
                context_id: pinned.query_context().clone(),
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from([a]),
                    relation_kind_scope: None,
                    max_depth: None,
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(917652),
                target_count_hint: 1,
            },
        )
        .unwrap();
    let c = create_entity("c");
    create_relation("bc", b, c);
    let answer = pinned.execute_query_plan(plan, &request).unwrap().unwrap();
    assert_eq!(
        answer
            .outcome
            .result
            .entities
            .iter()
            .map(|row| row.entity_id)
            .collect::<Vec<_>>(),
        vec![a, b]
    );
    assert_eq!(
        answer
            .outcome
            .result
            .relations
            .iter()
            .map(|row| row.relation_id)
            .collect::<Vec<_>>(),
        vec![ab]
    );
    assert_eq!(
        answer.outcome.plan.packet.context_id.version_id,
        VersionId(3)
    );
}

#[test]
fn exact_view_pin_keeps_original_root_after_commit_snapshot_release_and_rejects_foreign_runtime() {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let committed = create_entity_outcome(&request, &runtime, "original-view");
    let original_entity = changed_entities(&committed)[0];
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis.observation().commit_receipt(),
        Some(&committed.commit)
    );
    let view = runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    assert_eq!(
        runtime
            .snapshots()
            .retained_snapshot_for_commit(runtime.runtime_instance_id(), &committed.commit)
            .unwrap_err()
            .kind(),
        RelationalRetainedCommitSnapshotDenialKind::SnapshotNotRetained,
    );
    let foreign = memory_runtime();
    assert!(matches!(
        foreign
            .read_truth()
            .pin_view_read_capability(&view, &[], &request),
        Err(RelationalPinnedReadAdmissionDenial::SnapshotUnavailable),
    ));
    let pin = runtime
        .read_truth()
        .pin_view_read_capability(&view, &[], &request)
        .unwrap();
    assert_eq!(pin.read_context().version_id, committed.commit.version_id);
    let pinned_view = pin.project_read();
    let state_address = |view: &worth_relational::facade::runtime::VisibilityProjectionView| {
        view.read_exact_entity_state(&request, original_entity, |_, state| {
            std::ptr::from_ref(state.expect("the selected root carries its original payload"))
        })
        .unwrap()
        .unwrap()
    };
    assert_eq!(state_address(&pinned_view), state_address(&view));
    drop(pinned_view);
    let later = create_entity_outcome(&request, &runtime, "later-view");
    let later_entity = changed_entities(&later)[0];
    release_test_commit_snapshot(&runtime, &later);
    drop(later);
    assert_eq!(pin.read_context().version_id, committed.commit.version_id);
    drop(view);
    drop(basis);
    drop(committed);
    drop(runtime);
    drop(foreign);
    let projection = pin.project_read();
    for (entity, expected_present) in [(original_entity, true), (later_entity, false)] {
        let seen = projection
            .read_exact_entity_state(&request, entity, |record, state| {
                assert_eq!(record.entity_id, entity);
                let ContractValidatedAspectValueView::Scalar(AspectValue::String(
                    InternedString::Raw(text),
                )) = state
                    .expect("original payload")
                    .get(&aspect_key("name"))
                    .expect("original name aspect")
                    .view()
                else {
                    panic!("the original scalar payload must retain its plain string");
                };
                assert_eq!(text, "original-view");
            })
            .expect("the original request funds the exact pinned record");
        assert_eq!(
            seen.is_some(),
            expected_present,
            "the original pin excludes the later entity"
        );
    }
}
