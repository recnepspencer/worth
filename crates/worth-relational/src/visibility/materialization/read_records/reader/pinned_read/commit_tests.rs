use super::*;
use crate::facade::{
    config::CascadeDeletePolicy,
    history::BranchId,
    identity::{KindId, PartitionId},
};
use crate::indexes::data::{
    BoundedEntityFieldLookupDenialKind, BoundedIndexParityMode, DerivedIndexBuildRequest,
    DerivedIndexDiscardRequest, DerivedIndexKind,
};
use crate::query::data::*;
use crate::tests::support::*;
use crate::transactions::data::RecordRef;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

#[test]
fn pinned_read_keeps_rows_and_generation_across_a_commit_and_discard() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&request, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key("name"),
        CanonicalFieldPath::single(field_key("name")),
    );
    let definition = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(17),
        name: "commit-control".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: false,
    });
    let snapshot = runtime.visibility_authority().snapshot();
    let cold = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[definition.index_id], &request)
        .unwrap();
    let (_, admitted) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let build = runtime.index_authority().build_for_basis(
        &request,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![definition.index_id],
        },
        &admitted,
    );
    assert!(build.failed_indexes.is_empty());
    assert_eq!(
        cold.prepare_entity_field_lookup(definition.index_id, KindId(1), &locator, &request)
            .unwrap_err()
            .kind(),
        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
    );
    let read = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[definition.index_id], &request)
        .unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &snapshot,
            PlannedQueryPacket {
                label: "commit-control".into(),
                context_id: read.query_context().clone(),
                scope: QueryScope::ExplicitTargets {
                    targets: vec![RecordRef::Entity(entity)].into(),
                },
                locality: QueryLocalityClass::SinglePartition {
                    partition_id: entity.partition_id,
                },
                ordering: QueryOrderingContract::CanonicalRecordRefOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(91765),
                target_count_hint: 1,
            },
        )
        .unwrap();
    update_entity(&request, &runtime, entity, "after");
    runtime
        .index_authority()
        .discard_generations(DerivedIndexDiscardRequest::all_bases(definition.index_id))
        .unwrap();
    let late_view = runtime.read_truth().project_snapshot(&snapshot).unwrap();
    assert_eq!(
        runtime
            .index_access()
            .prepare_entity_field_lookup(
                &late_view,
                definition.index_id,
                KindId(1),
                &locator,
                &request
            )
            .unwrap_err()
            .kind(),
        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
    );
    let rows = read
        .execute_query_plan(plan, &request)
        .unwrap()
        .unwrap()
        .outcome
        .result
        .entities;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity_id, entity);
    assert_eq!(
        crate::visibility::materialization::read_records::entity_query_locus_value(
            &rows[0], &locator
        ),
        Some(&string_aspect_value("before"))
    );
    let prepared = read
        .prepare_entity_field_lookup(definition.index_id, KindId(1), &locator, &request)
        .unwrap();
    let before = prepared
        .execute_prepared_entity_field_lookup(
            &string_aspect_value("before"),
            2,
            BoundedIndexParityMode::Certification,
            &request,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(before.candidate_entity_ids(), &[entity]);
    assert_eq!(before.generation_id(), DerivedIndexGenerationId(1));
    let after = prepared
        .execute_prepared_entity_field_lookup(
            &string_aspect_value("after"),
            2,
            BoundedIndexParityMode::Production,
            &request,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert!(after.candidate_entity_ids().is_empty());
    assert_eq!(
        runtime
            .read_truth()
            .query_plan_context(&runtime.visibility_authority().snapshot())
            .unwrap()
            .version_id,
        crate::identity::data::VersionId(2)
    );
}

#[test]
fn pinned_traversal_keeps_its_frontier_across_a_commit() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&request, &runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&request, &runtime, "b", PartitionId(7));
    let ab = create_relation_in_partition(&request, &runtime, a, b, "ab", PartitionId(7));
    let snapshot = runtime.visibility_authority().snapshot();
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
    let c = create_entity_in_partition(&request, &runtime, "c", PartitionId(7));
    create_relation_in_partition(&request, &runtime, b, c, "bc", PartitionId(7));
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
        crate::identity::data::VersionId(3)
    );
}

#[test]
fn exact_view_pin_keeps_original_root_after_commit_snapshot_release_and_rejects_foreign_runtime() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
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
        runtime.snapshots().retained_snapshot_for_commit(runtime.runtime_instance_id(), &committed.commit)
            .unwrap_err().kind(),
        crate::visibility::exact_commit_snapshot::RelationalRetainedCommitSnapshotDenialKind::SnapshotNotRetained,
    );
    let foreign = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
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
    assert_eq!(pin.snapshot, ());
    assert_eq!(pin.context.version_id, committed.commit.version_id);
    assert!(Arc::ptr_eq(pin.basis.root(), view.selected_root().unwrap()));
    let later = create_entity_outcome(&request, &runtime, "later-view");
    let later_entity = changed_entities(&later)[0];
    release_test_commit_snapshot(&runtime, &later);
    drop(later);
    assert_eq!(pin.context.version_id, committed.commit.version_id);
    drop(view);
    drop(basis);
    drop(committed);
    drop(runtime);
    drop(foreign);
    use crate::visibility::materialization::read_records::projection::ProjectionRecordSource;
    use worth_foundational::facade::{
        AspectValue, ContractValidatedAspectValueView, InternedString,
    };
    let original = pin
        .authoritative_entity_record(&request, original_entity)
        .expect("original request reads the retained root")
        .expect("the original entity survives through its exact pin");
    assert_eq!(original.entity_id, original_entity);
    assert!(
        pin.authoritative_entity_record(&request, later_entity)
            .expect("original request reads the retained root")
            .is_none(),
        "the later entity is absent from the original root"
    );
    let state = original
        .authoritative_aspect_state
        .as_ref()
        .expect("original payload");
    let ContractValidatedAspectValueView::Struct(fields) = state
        .get(&aspect_key("name"))
        .expect("original name aspect")
        .view()
    else {
        panic!("the actual name aspect must be structured");
    };
    let Some(AspectValue::String(InternedString::Symbol(symbol))) = fields.get(&field_key("name"))
    else {
        panic!("the actual original payload must carry a present native Symbol");
    };
    assert_eq!(
        pin.symbols()
            .resolve(crate::symbols::data::Symbol(symbol.0)),
        Some("original-view")
    );
}
