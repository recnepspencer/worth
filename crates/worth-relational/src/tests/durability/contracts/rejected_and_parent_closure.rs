use super::*;

#[test]
fn durability_contract_recovery_ignores_rejected_relation_integrity_attempts() {
    let fixture = RelationIntegritySchemaFixture {
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
                pair_min_semantics:
                    crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement:
                    crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
            }],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        ..RelationIntegritySchemaFixture::default()
    };
    let store_layout = DurableStoreLayout {
        root_path: unique_test_store_path("worth-relational-rejected-relation-integrity-recovery"),
        segment_commit_capacity: 2,
    };
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(fixture.build_registry())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout.clone())
        .build();
    let source = create_entity(&runtime, "source");
    let target_a = create_entity(&runtime, "target-a");
    let target_b = create_entity(&runtime, "target-b");
    let accepted = create_relation_outcome(&runtime, source, target_a, "accepted");
    let relation = changed_relations(&accepted)[0];
    let latest_commit_before = runtime.history().latest_commit();
    let latest_patch_before = runtime.publication().latest_patch().unwrap().position;
    let main_digest_before = relation_aspect_history_digest_on_branch(
        &runtime,
        &BranchId("main".to_string()),
        relation,
        None,
    );

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("illegal-overflow").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("illegal-overflow"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target_b),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(error.code(), DiagnosticCode::RelationCardinalityViolation);
        }
        TransactionCommitError::Publication { .. } => {
            panic!("expected relation-integrity conflict, got publication error")
        }
        TransactionCommitError::Preparation { error, .. } => {
            panic!("expected relation-integrity conflict, got preparation error: {error:?}")
        }
        TransactionCommitError::Execution { denial, .. } => {
            panic!("expected relation-integrity conflict, got execution denial: {denial:?}")
        }
        TransactionCommitError::Interrupted { interruption, .. } => {
            panic!("expected relation-integrity conflict, got interruption: {interruption:?}")
        }
        TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. } => {
            panic!("expected relation-integrity conflict, got typed publication outcome")
        }
        TransactionCommitError::PerformedButDurabilityDeferred { .. } => {
            panic!("relation-integrity denial must happen before movement")
        }
    }

    assert_eq!(runtime.history().latest_commit(), latest_commit_before);
    assert_eq!(
        runtime.publication().latest_patch().unwrap().position,
        latest_patch_before
    );
    assert_eq!(
        relation_aspect_history_digest_on_branch(
            &runtime,
            &BranchId("main".to_string()),
            relation,
            None,
        ),
        main_digest_before
    );

    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(fixture.build_registry())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout)
        .build();
    let outcome = recovered.durability_recovery().recover(plan).unwrap();

    assert_eq!(outcome.latest_commit, latest_commit_before.clone());
    assert_eq!(
        relation_aspect_history_digest_on_branch(
            &recovered,
            &BranchId("main".to_string()),
            relation,
            None,
        ),
        main_digest_before
    );
    assert!(recovered
        .replay()
        .canonical_commit_envelope(latest_commit_before.unwrap().commit_id)
        .is_some());
}

#[test]
fn durability_contract_failure_missing_authoritative_parent_closure_is_explicit() {
    let runtime = runtime_with_test_schema();
    let parent = create_entity_outcome(&runtime, "main-a");
    let child = create_entity_outcome(&runtime, "main-b");
    let child_envelope = runtime
        .replay()
        .canonical_commit_envelope(child.commit.commit_id)
        .unwrap();
    let corrupt_plan = RecoveryPlan::new(
        runtime.config().clone(),
        runtime
            .config()
            .durability
            .policy
            .store_layout
            .clone()
            .map(|layout| DurableStore {
                layout,
                segments: Vec::new(),
                checkpoints: Vec::new(),
            }),
        None,
        None,
        vec![
            crate::durability::migration::ReadmittedCanonicalCommit::exact(
                crate::history::data::PositionedCanonicalCommit::for_test(
                    child.patch_position(),
                    std::sync::Arc::new(child_envelope),
                ),
            ),
        ],
        RecoveryCursor {
            checkpoint_id: None,
            segment_ids: Vec::new(),
        },
        RecoveryIntegrityReport {
            selected_checkpoint_id: None,
            skipped_corrupt_checkpoints: Vec::new(),
            verified_segment_ids: Vec::new(),
            corrupt_segment_id: None,
        },
        RecoveryAuthorityContinuityCheck::verified_at(ReplayVerificationLayer::DigestParity),
        RecoveryVerificationMode::NormalRecoveryVerification,
        DescriptorSemanticsVersion::default(),
        vec![child.commit.commit_id],
    );
    let mut recovered = runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(corrupt_plan)
        .unwrap_err();

    assert_eq!(parent.commit.commit_id.0, 1);
    assert_eq!(
        error.class,
        RecoveryFailureClass::MissingAuthoritativeParentClosure
    );
    assert_eq!(
        error.history_drift_class,
        Some(crate::facade::history::HistoryDriftClass::CanonicalHistoryDrift)
    );
}
