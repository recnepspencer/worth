use super::*;
use crate::facade::indexes::{
    DerivedIndexMaintenanceBudget, RelatedEntityEndpoint, RelatedEntityOrderingDirection,
    RelatedEntityOrderingField, RelationJoinDefinition, RelationJoinLeg,
    RelationJoinSharedEndpoint,
};

#[test]
fn retained_dangling_endpoint_deletion_matches_full_index_builds() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit);
    let parent = create_entity(execution, &runtime, "parent");
    let left = create_entity(execution, &runtime, "left");
    let shared = create_entity(execution, &runtime, "shared");
    let right = create_entity(execution, &runtime, "right");
    create_relation(execution, &runtime, parent, shared, "parent-shared");
    create_relation(execution, &runtime, left, shared, "left-shared");
    let before = create_relation_outcome(execution, &runtime, shared, right, "shared-right");
    let definitions = [
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "audit.ordering".into(),
            kind: DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: KindId(2),
                parent_endpoint: RelatedEntityEndpoint::SourceParent,
                child_kind: KindId(1),
                ordering: vec![RelatedEntityOrderingField::new(
                    aspect_field_locator(aspect_key("name"), field_key("name")),
                    RelatedEntityOrderingDirection::Ascending,
                )],
            },
            branch_scoped: true,
        },
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "audit.join".into(),
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Target, KindId(1)),
                RelationJoinLeg::new(KindId(2), RelationJoinSharedEndpoint::Source, KindId(1)),
                KindId(1),
            )),
            branch_scoped: true,
        },
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "audit.relation-field".into(),
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
            branch_scoped: true,
        },
    ];
    let ids = definitions
        .into_iter()
        .map(|definition| {
            runtime
                .index_authority()
                .register(definition)
                .expect("index fixture has identity capacity")
                .index_id
        })
        .collect::<Vec<_>>();
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: ids.clone(),
    };
    let built = runtime
        .index_authority()
        .build_for_commit(execution, request(before.commit.commit_id));
    assert!(built.failed_indexes.is_empty());
    let deleted = delete_entity(execution, &runtime, shared);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(deleted.commit.commit_id),
            &basis,
            Some(&before.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(100_000),
                maximum_cold_record_slots: Some(0),
                maximum_derived_rows: Some(100_000),
            },
            execution,
        )
        .unwrap();
    assert_eq!(local.work.cold_record_slots, 0);
    let rebuilt = runtime.index_authority().build_for_basis(
        execution,
        request(deleted.commit.commit_id),
        &basis,
    );
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(local.generations.len(), rebuilt.generations.len());
    for (incremental, cold) in local.generations.iter().zip(&rebuilt.generations) {
        assert_eq!(incremental.index_id, cold.index_id);
        assert_eq!(incremental.entries, cold.entries);
    }
    let child_changed = update_entity(execution, &runtime, right, "right-changed");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let local = runtime
        .index_authority()
        .refresh_for_basis(
            request(child_changed.commit.commit_id),
            &basis,
            Some(&deleted.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(100_000),
                maximum_cold_record_slots: Some(0),
                maximum_derived_rows: Some(100_000),
            },
            execution,
        )
        .unwrap();
    let rebuilt = runtime.index_authority().build_for_basis(
        execution,
        request(child_changed.commit.commit_id),
        &basis,
    );
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(local.generations.len(), rebuilt.generations.len());
    for (incremental, cold) in local.generations.iter().zip(&rebuilt.generations) {
        assert_eq!(incremental.index_id, cold.index_id);
        assert_eq!(incremental.entries, cold.entries);
    }
    release_test_commit_snapshot(&runtime, &before);
    release_test_commit_snapshot(&runtime, &deleted);
    release_test_commit_snapshot(&runtime, &child_changed);
}

#[test]
fn preflight_exact_projection_excludes_unprojected_kind_and_charges_audit_retained_relation() {
    use crate::config::data::CascadeDeletePolicy;
    use crate::facade::identity::{KindId, PartitionId};
    use crate::facade::indexes::DerivedIndexMaintenanceDenialKind as Denial;
    use crate::facade::schema::{
        EntityKindRegistration, KindAspectContractDeclarations, ProposedSchemaTransition,
        SchemaVersionId,
    };
    use crate::facade::transactions::{
        CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
    };

    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);

    // v1 schema: KindId(1) = Task (has `name`), KindId(3) = Project (NO `name`),
    // KindId(2) = ProjectTasks (RetainDanglingForAudit, has `label`).
    let mut fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let v1_registry = fixture
        .build_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "Project".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
        })
        .unwrap();
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(v1_registry)
        .build()
        .unwrap();

    // Seed 1 unprojected Project (KindId(3)), 3 Tasks (KindId(1)), and 3 relations;
    // then delete t3 so its relation becomes `RetainedDanglingForAudit`.
    let project_batch = WorkerIntentBatch::new("seed-project").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(3),
            client_key: crate::symbols::data::ClientKey::raw("p-main"),
            fields: crate::facade::transactions::AspectFieldPatch::default(),
        }),
    ));
    let mut project_txn = test_owner_begin_transaction_for_main(&runtime);
    project_txn
        .push_batch(
            project_batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let project_outcome = project_txn.commit(&execution, &runtime).unwrap();
    let project = changed_entities(&project_outcome)[0];
    release_test_commit_snapshot(&runtime, &project_outcome);

    let t1 = create_entity(&execution, &runtime, "t1");
    let t2 = create_entity(&execution, &runtime, "t2");
    let t3 = create_entity(&execution, &runtime, "t3");
    create_relation(&execution, &runtime, project, t1, "rel-1");
    create_relation(&execution, &runtime, project, t2, "rel-2");
    create_relation(&execution, &runtime, project, t3, "rel-3");
    delete_entity(&execution, &runtime, t3);

    // Prepare v2 target schema + 2 target indexes:
    // 1. EntityField on `name.name` (Task only — MUST NOT project Project!)
    // 2. RelationField on `label.label` (all 3 relations incl. RetainedDanglingForAudit!)
    fixture.schema_version_id = SchemaVersionId(2);
    let v2_registry = fixture
        .build_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "Project".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
        })
        .unwrap();

    let basis = test_owner_main_basis(&runtime).unwrap();
    let mut defs = runtime
        .index_authority()
        .prepare_home_program_definitions(&execution)
        .unwrap();
    defs.install(
        &execution,
        &DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "task.name".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: true,
        },
    )
    .unwrap();
    defs.install(
        &execution,
        &DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "rel.label".into(),
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
            branch_scoped: true,
        },
    )
    .unwrap();
    let prepared_defs = defs.finish(&execution).unwrap();
    let target = runtime
        .prepare_home_program_contracts(&execution, &basis, v2_registry, vec![], vec![])
        .unwrap()
        .with_index_definitions(prepared_defs)
        .unwrap();
    let transition = ProposedSchemaTransition {
        source_schema_id: fixture.schema_id.clone(),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: fixture.schema_id.clone(),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: Vec::new(),
    };

    // Independent exact Native counts:
    // - S = 4 entity slots (1 Project + 2 live Tasks + 1 deleted Task)
    //     + 3 relation slots (2 live + 1 RetainedDanglingForAudit)
    //     + 1 region = 8
    // - reads = 3 live entities + 3 relations (incl. RetainedDanglingForAudit) = 6
    // - R = 2 Task rows (Project NOT projected!) + 3 relation rows (incl. audit) = 5
    // - prepare publications = 2 indexes
    // - N = 8 + 6 + 5 + 2 = 21
    let exact = DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(21),
        maximum_cold_record_slots: Some(8),
        maximum_derived_rows: Some(5),
    };
    let work = runtime
        .preflight_home_program_target_index_budget(
            &execution,
            &basis,
            &target,
            Some((transition.clone(), None)),
            &[],
            exact,
        )
        .expect("exact independent (N=21, S=8, R=5) must admit without false refusal");
    assert_eq!(work.cold_record_slots, 8);
    assert_eq!(work.record_reads, 6);
    assert_eq!(work.derived_rows, 5);
    assert_eq!(work.generation_publications_reserved, 2);
    assert_eq!(work.work_units, 21);

    assert_eq!(
        runtime
            .preflight_home_program_target_index_budget(
                &execution,
                &basis,
                &target,
                Some((transition.clone(), None)),
                &[],
                DerivedIndexMaintenanceBudget {
                    maximum_cold_record_slots: Some(7),
                    ..exact
                },
            )
            .unwrap_err()
            .kind,
        Denial::ColdRecordSlotBudgetExceeded
    );
    assert_eq!(
        runtime
            .preflight_home_program_target_index_budget(
                &execution,
                &basis,
                &target,
                Some((transition.clone(), None)),
                &[],
                DerivedIndexMaintenanceBudget {
                    maximum_derived_rows: Some(4),
                    ..exact
                },
            )
            .unwrap_err()
            .kind,
        Denial::WorkBudgetExceeded
    );
    assert_eq!(
        runtime
            .preflight_home_program_target_index_budget(
                &execution,
                &basis,
                &target,
                Some((transition, None)),
                &[],
                DerivedIndexMaintenanceBudget {
                    maximum_work_units: Some(20),
                    ..exact
                },
            )
            .unwrap_err()
            .kind,
        Denial::WorkBudgetExceeded
    );
}
