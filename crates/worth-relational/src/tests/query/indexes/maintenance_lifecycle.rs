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
            unique_entity_kind: None,
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
            unique_entity_kind: None,
        },
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "audit.relation-field".into(),
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
            branch_scoped: true,
            unique_entity_kind: None,
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

    // Seed two identical runtimes so we can prove refused Create preflight
    // never advances live record/commit/version/transaction/proposal IDs.
    let build_seeded = || {
        let fixture = AspectSchemaFixture::with_default_declared_aspects(
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
        (fixture, runtime)
    };
    let (mut fixture, runtime) = build_seeded();
    let (_fixture_clean, runtime_clean) = build_seeded();

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
            unique_entity_kind: None,
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
            unique_entity_kind: None,
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

    let preflight = |before: &[&[WorkerIntentBatch]],
                     target_batches: &[WorkerIntentBatch],
                     budget: DerivedIndexMaintenanceBudget| {
        runtime.preflight_home_program_target_index_budget(
            &execution,
            &basis,
            &target,
            Some((transition.clone(), None)),
            before,
            target_batches,
            budget,
        )
    };

    // Exact counts: S=8 (4 entity + 3 relation + 1 region), reads=6, R=5, pub=2, N=21.
    let exact = DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(21),
        maximum_cold_record_slots: Some(8),
        maximum_derived_rows: Some(5),
    };
    let work = preflight(&[], &[], exact)
        .expect("exact independent (N=21, S=8, R=5) must admit without false refusal");
    assert_eq!(
        (
            work.cold_record_slots,
            work.record_reads,
            work.derived_rows,
            work.generation_publications_reserved,
            work.work_units,
        ),
        (8, 6, 5, 2, 21)
    );
    assert_eq!(
        preflight(
            &[],
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
        preflight(
            &[],
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
        preflight(
            &[],
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

    // Ordered BeforeTarget step creating a Task raises target S/reads/R/N:
    // S=9 (+1 entity slot), reads=7 (+1 live entity), R=6 (+1 Task row), N=24.
    let before_create = batch_create("t-before");
    let target_create = batch_create("t-target");
    let before_steps = [std::slice::from_ref(&before_create)];
    let before_work = preflight(
        &before_steps,
        &[],
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(24),
            maximum_cold_record_slots: Some(9),
            maximum_derived_rows: Some(6),
        },
    )
    .expect("BeforeTarget step creation must be accounted in target preflight");
    assert_eq!(
        (
            before_work.cold_record_slots,
            before_work.record_reads,
            before_work.derived_rows,
            before_work.work_units,
        ),
        (9, 7, 6, 24)
    );
    // Refused Create preflight (both BeforeTarget + Target) must leave live
    // entity, commit, version, transaction, and proposal IDs bit-for-bit intact.
    assert_eq!(
        preflight(&before_steps, std::slice::from_ref(&target_create), exact)
            .unwrap_err()
            .kind,
        Denial::ColdRecordSlotBudgetExceeded
    );
    let commit_live_task = |rt| {
        let mut tx = test_owner_begin_transaction_for_main(rt);
        let tx_id = tx.transaction_id();
        tx.push_batch(
            batch_create("after-preflight"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        let outcome = tx.commit(&execution, rt).unwrap();
        let ids = (
            tx_id,
            changed_entities(&outcome)[0],
            outcome.commit.commit_id,
            outcome.commit.version_id,
            rt.services.next_proposal_ordinal(),
        );
        release_test_commit_snapshot(rt, &outcome);
        ids
    };
    assert_eq!(commit_live_task(&runtime), commit_live_task(&runtime_clean));
}
