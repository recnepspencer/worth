use super::*;

#[test]
fn generated_group_suspends_and_rematerializes_with_exact_identity() {
    let runtime = runtime_with_test_schema();
    let (created, entity_fields, relation_fields) = create_generated_group(&runtime);
    let entities = changed_entities(&created);
    let relation = changed_relations(&created)[0];
    let original = runtime
        .read_truth()
        .read_snapshot(&created.snapshot)
        .expect("the source publication remains readable");
    let source_records = original.entities().to_vec();
    let source_relation = original.relations()[0].clone();

    let services = runtime.owner_component_services();
    let identity = runtime.main_branch_identity();
    let (_, source_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the source publication has an exact owner basis");
    let suspended = services
        .materialization_port()
        .suspend_generated_materialization(&source_basis, &entities)
        .expect("a complete all-create publication can suspend its payload");

    assert_eq!(suspended.custody.records().len(), 3);
    assert!(runtime
        .read_truth()
        .read_snapshot(&suspended.commit.snapshot)
        .is_none());
    let retained = runtime
        .read_truth()
        .read_snapshot(&created.snapshot)
        .expect("the retained source root remains independently readable");
    assert_eq!(retained.entities(), original.entities());
    assert_eq!(retained.relations(), original.relations());

    let (_, unavailable_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the suspended branch still has an owner-issued identity basis");
    assert_suspended_records(&unavailable_basis, &entities, relation);
    assert!(matches!(
        runtime
            .read_truth()
            .read_observation(&unavailable_basis.observation()),
        Err(crate::branch::RelationalBranchBasisDenial::MaterializationUnavailable)
    ));
    assert!(matches!(
        runtime
            .read_truth()
            .project_observation(&unavailable_basis.observation()),
        Err(crate::branch::RelationalBranchBasisDenial::MaterializationUnavailable)
    ));
    assert!(matches!(
        services
            .transaction_admission_port()
            .begin_branch_transaction(&unavailable_basis, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::MaterializationUnavailable)
    ));

    let failed = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable_basis,
            suspended.custody,
            vec![entity_materialization(
                entities[0],
                entity_fields[0].clone(),
            )],
            Vec::new(),
        )
        .expect_err("an incomplete reconstruction cannot consume custody");
    assert!(matches!(
        failed.error,
        RelationalMaterializationError::CandidateManifestMismatch
    ));

    let restored = services
        .materialization_port()
        .rematerialize_generated_materialization(
            &unavailable_basis,
            failed.custody,
            vec![
                entity_materialization(entities[0], entity_fields[0].clone()),
                entity_materialization(entities[1], entity_fields[1].clone()),
            ],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: entities[0],
                target: entities[1],
                fields: relation_fields,
            }],
        )
        .expect("the exact complete group rematerializes through owner authority");
    let restored_read = runtime
        .read_truth()
        .read_snapshot(&restored.snapshot)
        .expect("ordinary reads resume after complete reconstruction");

    assert_eq!(restored_read.entities(), source_records);
    assert_eq!(restored_read.relations(), &[source_relation]);
    let (_, restored_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the restored branch exposes a current owner basis");
    services
        .transaction_admission_port()
        .begin_branch_transaction(&restored_basis, RelationalTransactionIntent::ordinary())
        .expect("ordinary mutation admission resumes after reconstruction");
}

#[test]
fn ordinary_transaction_cannot_stage_an_owner_materialization_transition() {
    let runtime = runtime_with_test_schema();
    let created = crate::tests::support::create_entity_outcome(&runtime, "ordinary-denial");
    let entity = changed_entities(&created)[0];
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    let denial = transaction
        .push_batch(
            crate::transactions::data::WorkerIntentBatch::new("forged-materialization").push(
                MutationIntent::Materialization(MaterializationMutationIntent::SuspendEntity(
                    SuspendEntityMaterializationIntent { entity_id: entity },
                )),
            ),
        )
        .expect_err("ordinary mutation authority cannot suspend record materialization");

    assert_eq!(
        denial,
        crate::mvcc::RelationalTransactionStagingDenial::MaterializationAuthorityRequired
    );
}
