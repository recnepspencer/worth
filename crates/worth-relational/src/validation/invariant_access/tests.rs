use super::test_support::{
    create_entity, evaluate_main_commit_boundary_plan, evaluate_main_graph_composition_plan,
    relation_cardinality_runtime, relation_integrity_runtime,
    relation_integrity_runtime_with_scope_budget, relation_symmetry_runtime,
    runtime_with_invariants,
};
use crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection;
use crate::facade::identity::PartitionId;
use crate::facade::runtime::{
    InvariantCatalog, InvariantRegistration, InvariantRule, RelationalExecutionModel,
};
use crate::identity::data::KindId;
use crate::schema::data::SymmetryMode;
use crate::symbols::data::ClientKey;
use crate::tests::support::{aspect_key, field_key};
use crate::transactions::data::{
    BulkRelationCreateIntent, CreateIntent, DeleteRelationIntent, EntitySpec, MergedCommitPlan,
    MutationIntent, RelationMutationIntent, TransactionId,
};
use crate::validation::data::{
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantVerdict,
};
use crate::validation::engine::InvariantPlanScopeClass;

mod graph_composition_selection;

#[test]
fn commit_boundary_short_circuits_when_plan_contract_cannot_touch_profile_groups() {
    let runtime = runtime_with_invariants(
        InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
            )],
        },
        RelationalExecutionModel::SingleLaneExecution,
    );
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(1),
        merged_intents: vec![MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: crate::identity::data::RelationId::new(PartitionId::main(), 0, 1),
            },
        ))],
    };

    let results = evaluate_main_commit_boundary_plan(&runtime, &plan);

    assert!(results.results().is_empty());
}

#[test]
fn graph_composition_plan_uses_graph_composition_execution_profile() {
    let runtime = runtime_with_invariants(
        InvariantCatalog::default(),
        RelationalExecutionModel::SingleLaneExecution,
    );
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(11),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("graph-composition-profile"),
            fields: crate::transactions::data::AspectFieldPatch::default(),
        }))],
    };

    let results = evaluate_main_graph_composition_plan(&runtime, &plan);

    assert_eq!(
        results.metadata().execution_point(),
        InvariantExecutionPoint::GraphComposition
    );
    assert_eq!(results.metadata().max_cost(), InvariantCostClass::Touched);
    assert!(results.metadata().has_merged_plan());
}

#[test]
fn staged_parallel_commit_boundary_matches_serial_reference_results() {
    let invariant_catalog = InvariantCatalog {
        registrations: vec![
            InvariantRegistration::commit_boundary_blocking(
                InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
            ),
            InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(0)),
        ],
    };
    let serial_runtime = runtime_with_invariants(
        invariant_catalog.clone(),
        RelationalExecutionModel::SingleLaneExecution,
    );
    let staged_runtime = runtime_with_invariants(
        invariant_catalog,
        RelationalExecutionModel::ParallelPreparation,
    );
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(2),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("dup"),
            fields: crate::tests::support::single_string_aspect_field_patch(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
                "dup",
            ),
        }))],
    };

    let serial = evaluate_main_commit_boundary_plan(&serial_runtime, &plan);
    let staged = evaluate_main_commit_boundary_plan(&staged_runtime, &plan);

    assert_eq!(serial.results(), staged.results());
    assert_eq!(
        serial.summary().result_count(),
        staged.summary().result_count()
    );
    assert_eq!(
        staged
            .metadata()
            .preparation_strategy()
            .map(|strategy| strategy.selected_mode),
        Some(PreparationStrategySelection::StagedParallel)
    );
    assert!(staged.results().iter().any(|result| {
        result.failure_effect == InvariantFailureEffect::BlockCommit
            && matches!(result.verdict, InvariantVerdict::Violation(_))
    }));
}

#[test]
fn commit_boundary_metadata_exposes_proof_boundary_summary_for_packet_backed_execution() {
    let runtime = relation_integrity_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(3),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("planned"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))],
    };

    let result = evaluate_main_commit_boundary_plan(&runtime, &plan);
    let summary = result
        .metadata()
        .proof_boundary()
        .expect("proof boundary summary");

    assert_eq!(
        summary.scope_class(),
        InvariantPlanScopeClass::PartitionScope
    );
    assert!(summary.widened_causes().is_empty());
    assert_eq!(summary.packet_count(), 1);
    assert_eq!(summary.touched_partition_count(), 1);
}

#[test]
fn commit_boundary_symmetry_failure_fields_localize_missing_twin_endpoints() {
    let runtime = relation_symmetry_runtime(SymmetryMode::PairedTwinRequired);
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(4),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("missing-twin"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))],
    };

    let result = evaluate_main_commit_boundary_plan(&runtime, &plan);
    let failure = result
        .summary()
        .blocking_failure()
        .expect("blocking symmetry failure");
    assert_eq!(
        failure.violation().code,
        crate::diagnostics::data::DiagnosticCode::RelationSymmetryViolation
    );
    match failure.fields() {
        crate::validation::data::InvariantViolationFields::RelationSymmetry {
            contract_id,
            relation_kind_id,
            source: actual_source,
            target: actual_target,
            mode,
        } => {
            assert_eq!(contract_id.as_str(), "paired_twin");
            assert_eq!(*relation_kind_id, KindId(2));
            assert_eq!(
                *actual_source,
                crate::transactions::data::EntityReference::Existing(source)
            );
            assert_eq!(
                *actual_target,
                crate::transactions::data::EntityReference::Existing(target)
            );
            assert_eq!(*mode, SymmetryMode::PairedTwinRequired);
        }
        fields => panic!("expected typed symmetry fields, got {fields:?}"),
    }
}

#[test]
fn commit_boundary_cardinality_failure_fields_localize_nonmanifold_like_overflow() {
    let runtime = relation_cardinality_runtime();
    let source = create_entity(&runtime, "source");
    let target_a = create_entity(&runtime, "target-a");
    let target_b = create_entity(&runtime, "target-b");
    let _accepted = {
        let mut txn = {
            let transaction_validation_input =
                crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime);
            runtime
                .begin_branch_transaction(
                    transaction_validation_input.basis(),
                    transaction_validation_input.intent().clone(),
                )
                .expect("owner-admitted transaction context")
        };
        txn.push_batch(
            crate::facade::transactions::WorkerIntentBatch::new("accepted").push(
                MutationIntent::Create(CreateIntent::Relation(
                    crate::transactions::data::RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(2),
                        client_key: ClientKey::raw("accepted"),
                        source: crate::transactions::data::EntityReference::Existing(source),
                        target: crate::transactions::data::EntityReference::Existing(target_a),
                        fields: crate::transactions::data::AspectFieldPatch::default(),
                    },
                )),
            ),
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(&runtime).unwrap()
    };
    let overflow_plan = MergedCommitPlan {
        transaction_id: TransactionId(5),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("overflow"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target_b),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))],
    };

    let result = evaluate_main_commit_boundary_plan(&runtime, &overflow_plan);
    let failure = result
        .summary()
        .blocking_failure()
        .expect("blocking cardinality failure");
    assert_eq!(
        failure.violation().code,
        crate::diagnostics::data::DiagnosticCode::RelationCardinalityViolation
    );
    match failure.fields() {
        crate::validation::data::InvariantViolationFields::RelationCardinalityEndpoint {
            contract_id,
            relation_kind_id,
            entity_id,
            boundary,
            count,
            limit,
        } => {
            assert_eq!(contract_id.as_str(), "source_max_one");
            assert_eq!(*relation_kind_id, KindId(2));
            assert_eq!(
                *entity_id,
                crate::transactions::data::EntityReference::Existing(source)
            );
            assert_eq!(
                *boundary,
                crate::validation::data::RelationCardinalityBoundary::Source
            );
            assert_eq!(*count, 2);
            assert_eq!(*limit, 1);
        }
        fields => panic!("expected typed cardinality endpoint fields, got {fields:?}"),
    }
}

#[test]
fn commit_boundary_reports_relation_integrity_scope_budget_violation_as_blocking_failure() {
    let runtime = relation_integrity_runtime_with_scope_budget(
        crate::config::data::RelationIntegrityScopeBudget {
            max_relation_kinds: 8,
            max_touched_entities: 16,
            max_deleted_entities: 8,
            max_scanned_relations: 16,
            max_planned_edges: 1,
        },
    );
    let source_a = create_entity(&runtime, "source-a");
    let target_a = create_entity(&runtime, "target-a");
    let source_b = create_entity(&runtime, "source-b");
    let target_b = create_entity(&runtime, "target-b");
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(6),
        merged_intents: vec![MutationIntent::Create(CreateIntent::BulkRelations(
            BulkRelationCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_keys: vec![ClientKey::raw("edge-a"), ClientKey::raw("edge-b")],
                endpoints: vec![
                    (
                        crate::transactions::data::EntityReference::Existing(source_a),
                        crate::transactions::data::EntityReference::Existing(target_a),
                    ),
                    (
                        crate::transactions::data::EntityReference::Existing(source_b),
                        crate::transactions::data::EntityReference::Existing(target_b),
                    ),
                ],
                field_patches: vec![
                    crate::transactions::data::AspectFieldPatch::default(),
                    crate::transactions::data::AspectFieldPatch::default(),
                ],
            },
        ))],
    };

    let result = evaluate_main_commit_boundary_plan(&runtime, &plan);
    let failure = result
        .summary()
        .blocking_failure()
        .expect("blocking scope budget failure");
    assert_eq!(
        failure.code(),
        crate::diagnostics::data::DiagnosticCode::PreparationFailure
    );
    match failure.fields() {
        crate::validation::data::InvariantViolationFields::RelationIntegrityScopeBudgetExceeded {
            limit_name,
            limit,
            observed,
            planned_edge_count,
            ..
        } => {
            assert_eq!(limit_name, "max_planned_edges");
            assert_eq!(*limit, 1);
            assert_eq!(*observed, 2);
            assert_eq!(*planned_edge_count, 2);
        }
        fields => panic!("expected typed scope budget fields, got {fields:?}"),
    }
}
