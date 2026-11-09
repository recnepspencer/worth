//! Restoration must judge the restored edge against live outside constraints.
use super::*;
use worth_relational::facade::mvcc::RelationalOwnedGraphSelectionPreparationError;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
#[test]
fn graph_selection_rejudges_the_selected_target_with_the_real_cardinality_contract() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let fixture = AspectSchemaFixture::default();
    let registry = registry()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(7),
            kind_name: "ExclusiveReference".into(),
            schema_id: fixture.schema_id,
            schema_version_id: fixture.schema_version_id,
            cross_context_policy:
                worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
            cascade_delete_policy:
                worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![],
                vec![CardinalityContractDeclaration {
                    contract_id: ContractId::new("ExclusiveReference:target"),
                    source_max: None,
                    target_max: Some(1),
                    pair_max: None,
                    source_min: None,
                    target_min: None,
                    pair_min: None,
                    pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
                    minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
                }],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            RelationalSchemaRegistry::new(),
            vec![ownership().invariant_registration().unwrap()],
        )
        .unwrap();
    let model = create_entity(execution, &runtime, "model");
    let outside = create_entity_in_partition(execution, &runtime, "outside", PartitionId(7));
    let catalog = create_entity_in_partition(execution, &runtime, "catalog", PartitionId(5));
    let other = create_entity_in_partition(execution, &runtime, "other-catalog", PartitionId(6));
    let member = member(execution, &runtime, "member");
    create_relation(execution, &runtime, model, member, "owns");
    let reference =
        create_relation_of_kind(execution, &runtime, member, catalog, "exclusive", KindId(7));
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut tx = begin(&runtime);
    tx.push_batch(
        WorkerIntentBatch::new("move-reference").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: reference,
                kind_id: KindId(7),
                source: EntityReference::Existing(member),
                target: EntityReference::Existing(other),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let changed = tx.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &changed);
    let foreign = create_relation_of_kind(
        execution,
        &runtime,
        outside,
        catalog,
        "foreign-exclusive",
        KindId(7),
    );
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let graph = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &current,
            &selected,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap();
    let input = runtime
        .preparation_port()
        .admit_owned_graph_selection(execution, begin(&runtime), graph)
        .unwrap();
    let error = runtime
        .preparation_port()
        .prepare_owned_graph_selection(execution, input, None)
        .unwrap_err();
    let RelationalOwnedGraphSelectionPreparationError::Preparation(
        TransactionCommitError::Conflict { error, .. },
    ) = error
    else {
        panic!("{error:?}");
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id,
                relation_kind_id,
                entity_id,
                count,
                limit,
                ..
            },
        ..
    } = error.class
    else {
        panic!("{error:?}");
    };
    assert_eq!(contract_id, ContractId::new("ExclusiveReference:target"));
    assert_eq!(relation_kind_id, KindId(7));
    assert_eq!(entity_id, EntityReference::Existing(catalog));
    assert_eq!(count, 2);
    assert_eq!(limit, 1);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        current.descriptor()
    );
    assert_eq!(
        begin(&runtime)
            .read_relation(execution, foreign)
            .unwrap()
            .base()
            .unwrap()
            .source,
        outside
    );
    assert_eq!(
        begin(&runtime)
            .read_relation(execution, reference)
            .unwrap()
            .base()
            .unwrap()
            .target,
        other
    );
}
