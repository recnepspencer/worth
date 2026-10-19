//! Schema-classified empty entities keep actual native incidence and custody.
use super::*;

#[test]
fn fieldless_birth_incidence_survives_warm_no_data_preserve_and_cold_readmission() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let fixture = AspectSchemaFixture::default();
    let mut runtime = fixture.build_runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let targets = ["fieldless-region", "fieldless-shell"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let mut batch = WorkerIntentBatch::new("actual-fieldless-birth");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: AspectFieldPatch::default(),
        })));
    }
    let edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("actual-incidence"),
        source: EntityReference::Created(targets[0].clone()),
        target: EntityReference::Created(targets[1].clone()),
    };
    batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: edge.partition_id,
            kind_id: edge.kind_id,
            client_key: edge.client_key.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
            fields: AspectFieldPatch::default(),
        },
    )));
    let birth = publish(
        &runtime,
        execution,
        &installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("native-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| birth.created_entity(&target).unwrap());
    let edge_id = birth.created_relation(&edge).unwrap();
    let born_at = birth.commit.commit_id;
    let born_version = birth.commit.version_id;
    let envelope = &birth.publication().envelope;
    assert_eq!(envelope.record_allocations.len(), 3);
    assert_eq!(envelope.patch.authoritative_record_patches.len(), 3);
    for patch in &envelope.patch.authoritative_record_patches {
        assert_eq!(
            patch.structural_change,
            crate::publication::patch::data::RecordStructuralChange::Created
        );
        assert!(!patch.contains_opaque_aspect);
        assert_eq!(patch.authoritative_patch.full_grammar_operation_count(), 0);
    }
    let previous = birth.required_output_publication(&installed).unwrap();
    let origin = previous.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.outputs, ids);
    assert_eq!(origin.members.len(), 2);
    assert_eq!(origin.relations.len(), 1);
    assert_eq!(origin.relations[0].id, edge_id);
    assert_eq!(origin.relations[0].source, ids[0]);
    assert_eq!(origin.relations[0].target, ids[1]);
    release_test_commit_snapshot(&runtime, &birth);
    let preserved = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("genuine-fieldless-no-data-preserve"),
        preserve(&owner, previous, ids, 2),
    )
    .unwrap();
    assert!(preserved.changed_records.is_empty());
    assert_ne!(preserved.commit.commit_id, born_at);
    let preserved_at = preserved.commit.commit_id;
    let warm = preserved.required_output_publication(&installed).unwrap();
    let origin = warm.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.outputs, ids);
    assert_eq!(origin.birth, born_at);
    assert_eq!(origin.relations.len(), 1);
    assert_eq!(origin.relations[0].id, edge_id);
    for member in &origin.members {
        assert_eq!(member.posture, RequiredOutputRolePosture::Preserve);
        assert_eq!(member.birth, born_at);
        assert_eq!(member.after_write, born_version);
    }
    release_test_commit_snapshot(&runtime, &preserved);
    drop(warm);
    let saved = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let mut cold = fixture.build_runtime();
    let receipt = cold
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &saved)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    let basis = test_owner_main_basis(&cold).unwrap();
    let partition = basis
        .inner
        .root
        .partition_state(PartitionId::main())
        .unwrap();
    let plan = basis
        .inner
        .root
        .schema_authority()
        .entity_aspect_plan(KindId(1))
        .unwrap();
    assert!(plan.executable_bindings.is_empty());
    for id in ids {
        let row = partition.entity_arena.get(&id).unwrap();
        assert!(row.is_live());
        assert_eq!(row.generation(), id.generation_value());
        assert_eq!(row.kind_id(), Some(KindId(1)));
        assert!(row.extra().authoritative_aspect_state.is_none());
    }
    let readmitted = cold
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-preserver",
            &[2; 32],
            &crate::mvcc::native_meaning_fixture("native-creator"),
            &crate::mvcc::native_roles_fixture(
                &ids.map(|id| (id, KindId(1))),
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .unwrap();
    assert_eq!(readmitted.envelope.commit.commit_id, preserved_at);
    let origin = readmitted.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.outputs, ids);
    assert_eq!(origin.relations.len(), 1);
    assert_eq!(origin.relations[0].id, edge_id);
    assert_eq!(origin.relations[0].source, ids[0]);
    assert_eq!(origin.relations[0].target, ids[1]);

    // The same schema classification grants no producer ownership to ordinary work.
    let ordinary = create_entity_outcome(execution, &cold, "ordinary-fieldless");
    let ordinary_id = changed_entities(&ordinary)[0];
    assert!(ordinary.required_output_publication(&installed).is_none());
    release_test_commit_snapshot(&cold, &ordinary);
    let basis = test_owner_main_basis(&cold).unwrap();
    assert!(cold
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-creator",
            &[1; 32],
            &crate::mvcc::native_meaning_fixture("native-creator"),
            &crate::mvcc::native_roles_fixture(
                &[(ordinary_id, KindId(1))],
                RequiredOutputRolePosture::Create,
            ),
        )
        .is_err());
}
