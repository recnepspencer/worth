use super::*;

#[test]
fn replay_contract_preserves_relation_integrity_declared_schema() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::new(
                    Vec::new(),
                    vec![crate::schema::data::CardinalityContractDeclaration {
                        contract_id: "source_max_one".into(),
                        source_max: Some(1),
                        source_min: None,
                        target_max: None,
                        target_min: None,
                        pair_max: None,
                        pair_min: None,
                        pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement:
                            crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                    }],
                    vec![crate::schema::data::UniquenessContractDeclaration {
                        contract_id: "uniq".into(),
                        scope: crate::schema::data::UniquenessScope::DirectedSemanticEdge,
                    }],
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let outcome = create_relation_outcome(&runtime, source, target, "guarded");

    let replay = runtime
        .replay_authority()
        .replay_commit(RelationalReplayRequest {
            commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
        });
    let replay_access = runtime.replay();
    let envelope = replay_access
        .canonical_commit_envelope(outcome.commit.commit_id)
        .unwrap();

    assert!(runtime.replay().compare_outcome(&replay));
    let relation_authority = envelope
        .schema_authority
        .relation_kinds
        .iter()
        .find(|kind| kind.kind_id == KindId(2))
        .expect("relation schema authority");
    assert_eq!(
        relation_authority.aspect_plan_revision,
        runtime
            .schema_registry()
            .relation_registration(KindId(2))
            .unwrap()
            .aspect_contract_declarations
            .plan_revision
    );
    assert_eq!(
        relation_authority.relation_integrity_plan_revision,
        runtime
            .schema_registry()
            .relation_registration(KindId(2))
            .unwrap()
            .relation_integrity
            .plan_revision
    );
}

#[test]
fn replay_contract_preserves_branch_local_relation_integrity_truth_after_rejected_feature_attempt()
{
    let runtime = source_max_one_relation_integrity_runtime();
    let source = create_entity(&runtime, "source");
    let target_a = create_entity(&runtime, "target-a");
    let target_b = create_entity(&runtime, "target-b");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();

    let accepted_feature = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
            &runtime,
            BranchId("feature".to_string()),
        );
        txn.push_batch(
            WorkerIntentBatch::new("accepted-feature-relation").push(MutationIntent::Create(
                CreateIntent::Relation(crate::transactions::data::RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("feature-accepted"),
                    source: crate::transactions::data::EntityReference::Existing(source),
                    target: crate::transactions::data::EntityReference::Existing(target_a),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    };
    let feature_head_before_reject = runtime
        .history()
        .branch_head(&BranchId("feature".to_string()));

    let mut rejected_txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("feature".to_string()),
    );
    rejected_txn
        .push_batch(
            WorkerIntentBatch::new("rejected-feature-relation").push(MutationIntent::Create(
                CreateIntent::Relation(crate::transactions::data::RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw("feature-rejected"),
                    source: crate::transactions::data::EntityReference::Existing(source),
                    target: crate::transactions::data::EntityReference::Existing(target_b),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let rejected = rejected_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();

    match rejected {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(error.code(), DiagnosticCode::RelationCardinalityViolation);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("feature".to_string())),
        feature_head_before_reject
    );

    let replay = runtime
        .replay_authority()
        .replay_commit(RelationalReplayRequest {
            commit_id: accepted_feature.commit.commit_id,
            branch_id: BranchId("feature".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
        });

    assert!(runtime.replay().compare_outcome(&replay));
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::History));
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("feature".to_string()))
            .unwrap()
            .commit_id,
        accepted_feature.commit.commit_id
    );
}
