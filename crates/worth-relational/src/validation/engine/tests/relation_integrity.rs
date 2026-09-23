use super::validation_engine_fixtures::*;

#[test]
fn engine_rejects_cross_partition_relations_under_partition_isolation_contracts() {
    let runtime = runtime_with_partition_isolation();
    let source = crate::identity::data::EntityId::new(PartitionId(1), 0, 1);
    let target = crate::identity::data::EntityId::new(PartitionId(2), 0, 1);
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(11),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-a"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))],
    };

    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            Some(crate::validation::data::InvariantPlanContract::from_merged_plan(&plan)),
        ),
    );

    let failure = results
        .results()
        .iter()
        .find_map(|result| match &result.verdict {
            crate::validation::data::InvariantVerdict::Violation(violation) => Some(violation),
            _ => None,
        })
        .expect("partition isolation violation");
    match &failure.fields {
        InvariantViolationFields::PartitionIsolation {
            contract_id,
            source_partition_id,
            target_partition_id,
            ..
        } => {
            assert_eq!(contract_id.as_str(), "same_partition");
            assert_eq!(*source_partition_id, PartitionId(1));
            assert_eq!(*target_partition_id, PartitionId(2));
        }
        other => panic!("expected partition isolation violation, got {other:?}"),
    }
}

#[test]
fn engine_rejects_planned_cycles_under_acyclicity_contracts() {
    let runtime = runtime_with_acyclicity_and_connectivity();
    let a = crate::identity::data::EntityId::new(PartitionId::main(), 0, 1);
    let b = crate::identity::data::EntityId::new(PartitionId::main(), 1, 1);
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(12),
        merged_intents: vec![
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-ab"),
                source: crate::transactions::data::EntityReference::Existing(a),
                target: crate::transactions::data::EntityReference::Existing(b),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            })),
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-ba"),
                source: crate::transactions::data::EntityReference::Existing(b),
                target: crate::transactions::data::EntityReference::Existing(a),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            })),
        ],
    };

    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            Some(crate::validation::data::InvariantPlanContract::from_merged_plan(&plan)),
        ),
    );

    let failure = results
        .results()
        .iter()
        .find_map(|result| match &result.verdict {
            crate::validation::data::InvariantVerdict::Violation(violation) => Some(violation),
            _ => None,
        })
        .expect("acyclicity violation");
    match &failure.fields {
        InvariantViolationFields::Acyclicity { contract_id, .. } => {
            assert_eq!(contract_id.as_str(), "no_cycles");
        }
        other => panic!("expected acyclicity violation, got {other:?}"),
    }
}

#[test]
fn prepared_acyclicity_scope_rejects_visible_graphs_that_exceed_scan_budget() {
    let mut runtime =
        runtime_with_acyclicity_and_connectivity_budget(RelationIntegrityScopeBudget {
            max_relation_kinds: 8,
            max_touched_entities: 16,
            max_deleted_entities: 8,
            max_scanned_relations: 16,
            max_planned_edges: 8,
        });
    let a = create_entity_of_kind(&runtime, KindId(3), "a");
    let b = create_entity_of_kind(&runtime, KindId(3), "b");
    let c = create_entity_of_kind(&runtime, KindId(3), "c");
    let d = create_entity_of_kind(&runtime, KindId(3), "d");
    let e = create_entity_of_kind(&runtime, KindId(3), "e");
    create_relation_of_kind(&runtime, KindId(2), a, b, "edge-ab");
    create_relation_of_kind(&runtime, KindId(2), b, c, "edge-bc");
    create_relation_of_kind(&runtime, KindId(2), c, d, "edge-cd");
    create_relation_of_kind(&runtime, KindId(2), d, e, "edge-de");
    runtime.configure_for_test(|config| {
        config
            .execution
            .relation_integrity_scope_budget
            .max_scanned_relations = 2
    });

    let plan = MergedCommitPlan {
        transaction_id: TransactionId(19),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-ea"),
                source: crate::transactions::data::EntityReference::Existing(e),
                target: crate::transactions::data::EntityReference::Existing(a),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))],
    };

    let result =
        crate::validation::invariant_access::test_support::evaluate_main_commit_boundary_plan(
            &runtime, &plan,
        );
    let failure = result
        .summary()
        .blocking_failure()
        .expect("prepared scope budget violation");
    match failure.fields() {
        InvariantViolationFields::RelationIntegrityScopeBudgetExceeded {
            limit_name,
            limit,
            observed,
            ..
        } => {
            assert_eq!(limit_name, "max_scanned_relations");
            assert_eq!(*limit, 2);
            assert_eq!(*observed, 3);
        }
        other => panic!("expected traversal budget violation, got {other:?}"),
    }
}

#[test]
fn commit_publication_stage_rejects_sources_without_required_connectivity() {
    let runtime = runtime_with_acyclicity_and_connectivity();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("node-a").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("node-a"),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(&runtime)
        .expect_err("connectivity publication failure");
    match error {
        crate::facade::transactions::TransactionCommitError::Publication { error, .. } => {
            assert!(error.detail.contains("reachable_anchor"));
            assert!(error.detail.contains("at least 1 reachable target"));
        }
        other => panic!("expected publication error, got {other:?}"),
    }
}

#[test]
fn minimum_cardinality_current_version_scans_only_live_slots() {
    let runtime = runtime_with_cardinality_minimum();
    let source = create_entity_of_kind(&runtime, KindId(1), "source");
    let target = create_entity_of_kind(&runtime, KindId(1), "target");
    let retired_target = create_entity_of_kind(&runtime, KindId(1), "retired-target");
    let retired_relation =
        create_relation_of_kind(&runtime, KindId(2), source, retired_target, "retired");
    create_relation_of_kind(&runtime, KindId(2), source, target, "live");
    let mut delete_txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    delete_txn
        .push_batch(
            WorkerIntentBatch::new("delete-retired").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: retired_relation,
                }),
            )),
        )
        .expect("test staging stays within configured resource budgets");
    delete_txn.commit(&runtime).expect("retire relation");

    runtime.performance_access().reset_counters();
    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CertificationBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );

    let violating_contracts = results
        .results()
        .iter()
        .filter_map(|result| match (&result.rule, &result.verdict) {
            (
                crate::validation::data::InvariantReportedRule::Native(
                    crate::validation::data::InvariantRule::CardinalityMinimumContract(contract),
                ),
                crate::validation::data::InvariantVerdict::Violation(_),
            ) => Some(contract.contract_id.as_str()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(violating_contracts, ["min_one", "min_one_again"].into());

    let counters = runtime.performance_access().counters();
    assert_eq!(
        counters.relation_cardinality_minimum_certification_contracts_evaluated,
        2,
    );
    assert_eq!(
        counters.relation_cardinality_minimum_certification_relation_slot_scans,
        1
    );
    assert_eq!(
        counters.relation_cardinality_minimum_certification_entity_slot_scans,
        3
    );
}

#[test]
fn minimum_cardinality_shared_index_preserves_both_passing_contracts() {
    let runtime = runtime_with_cardinality_minimum();
    let first = create_entity_of_kind(&runtime, KindId(1), "first");
    let second = create_entity_of_kind(&runtime, KindId(1), "second");
    create_relation_of_kind(&runtime, KindId(2), first, second, "forward");
    create_relation_of_kind(&runtime, KindId(2), second, first, "reverse");

    runtime.performance_access().reset_counters();
    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CertificationBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );
    let passing_contracts = results
        .results()
        .iter()
        .filter_map(|result| match (&result.rule, &result.verdict) {
            (
                crate::validation::data::InvariantReportedRule::Native(
                    crate::validation::data::InvariantRule::CardinalityMinimumContract(contract),
                ),
                crate::validation::data::InvariantVerdict::Pass,
            ) => Some(contract.contract_id.as_str()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(passing_contracts, ["min_one", "min_one_again"].into());
    let counters = runtime.performance_access().counters();
    assert_eq!(
        counters.relation_cardinality_minimum_certification_contracts_evaluated,
        2,
    );
    assert_eq!(
        counters.relation_cardinality_minimum_certification_relation_slot_scans,
        2,
    );
    assert_eq!(
        counters.relation_cardinality_minimum_certification_entity_slot_scans,
        2,
    );
}
