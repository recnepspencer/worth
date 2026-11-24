use super::validation_engine_fixtures::*;
use crate::config::data::RelationalRuntimeConfig;
use crate::identity::data::VersionId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::storage::overlay::small_relation_partition::{
    relation_partition_for_test, typed_relation_partition_for_test,
};
use crate::storage::overlay::PartitionState;
use std::collections::BTreeMap;
use worth_relational_storage::internal_port::storage::partition::adjacency::AdjacencySetFixturePort as _;

fn evaluate(
    schema: RelationalSchemaRegistry,
    partitions: &BTreeMap<PartitionId, PartitionState>,
    version: VersionId,
    profile: InvariantRequestProfile,
    plan: Option<&MergedCommitPlan>,
    instrumentation: &RuntimeInstrumentation,
) -> crate::validation::engine::InvariantExecutionResult {
    let mut config = RelationalRuntimeConfig::default();
    config.schema.registry = schema;
    let view =
        crate::validation::engine::test_view::invariant_view(instrumentation, config, version);
    InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            profile,
            &view,
            InvariantObservation::committed_branch(partitions),
            version,
            plan,
            plan.map(crate::validation::data::contracts::InvariantPlanContract::from_merged_plan),
        ),
    )
}

#[test]
fn engine_rejects_cross_partition_relations_under_partition_isolation_contracts() {
    let source = crate::identity::data::EntityId::new(PartitionId(1), 0, 1);
    let target = crate::identity::data::EntityId::new(PartitionId(2), 0, 1);
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(11),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-a"),
                source: crate::transactions::data::primitives::EntityReference::Existing(source),
                target: crate::transactions::data::primitives::EntityReference::Existing(target),
                fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
            },
        ))],
    };

    let results = evaluate(
        partition_isolation_registry(),
        &BTreeMap::new(),
        VersionId(0),
        InvariantRequestProfile::CommitBoundary,
        Some(&plan),
        &RuntimeInstrumentation::new(),
    );

    let failure = results
        .results()
        .iter()
        .find_map(|result| match &result.verdict {
            crate::validation::data::execution::InvariantVerdict::Violation(violation) => {
                Some(violation)
            }
            _ => None,
        })
        .expect("partition isolation violation");
    match &failure.fields {
        InvariantViolationFields::PartitionIsolation {
            contract_id,
            source_partition_id,
            target_partition_id,
            source: actual_source,
            target: actual_target,
            ..
        } => {
            assert_eq!(contract_id.as_str(), "same_partition");
            assert_eq!(*source_partition_id, PartitionId(1));
            assert_eq!(*target_partition_id, PartitionId(2));
            assert_eq!(
                actual_source,
                &crate::transactions::data::primitives::EntityReference::Existing(source)
            );
            assert_eq!(
                actual_target,
                &crate::transactions::data::primitives::EntityReference::Existing(target)
            );
        }
        other => panic!("expected partition isolation violation, got {other:?}"),
    }
}

#[test]
fn engine_rejects_planned_cycles_under_acyclicity_contracts() {
    let a = crate::identity::data::EntityId::new(PartitionId::main(), 0, 1);
    let b = crate::identity::data::EntityId::new(PartitionId::main(), 1, 1);
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(12),
        merged_intents: vec![
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-ab"),
                source: crate::transactions::data::primitives::EntityReference::Existing(a),
                target: crate::transactions::data::primitives::EntityReference::Existing(b),
                fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
            })),
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-ba"),
                source: crate::transactions::data::primitives::EntityReference::Existing(b),
                target: crate::transactions::data::primitives::EntityReference::Existing(a),
                fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
            })),
        ],
    };

    let results = evaluate(
        acyclicity_and_connectivity_registry(),
        &BTreeMap::new(),
        VersionId(0),
        InvariantRequestProfile::CommitBoundary,
        Some(&plan),
        &RuntimeInstrumentation::new(),
    );

    let failure = results
        .results()
        .iter()
        .find_map(|result| match &result.verdict {
            crate::validation::data::execution::InvariantVerdict::Violation(violation) => {
                Some(violation)
            }
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
fn prepared_acyclicity_scope_follows_all_visible_edges_to_late_cycle() {
    let partition_id = PartitionId::main();
    let (partition, nodes, _) = typed_relation_partition_for_test(
        partition_id,
        [KindId(3); 5].into_iter(),
        &[
            (0, 1, KindId(2)),
            (1, 2, KindId(2)),
            (2, 3, KindId(2)),
            (3, 4, KindId(2)),
        ],
    );
    let a = nodes[0];
    let e = nodes[4];
    let partitions = BTreeMap::from([(partition_id, partition)]);
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(19),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-ea"),
                source: crate::transactions::data::primitives::EntityReference::Existing(e),
                target: crate::transactions::data::primitives::EntityReference::Existing(a),
                fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
            },
        ))],
    };

    let result = evaluate(
        acyclicity_and_connectivity_registry(),
        &partitions,
        VersionId(1),
        InvariantRequestProfile::CommitBoundary,
        Some(&plan),
        &RuntimeInstrumentation::new(),
    );
    let failure = result
        .summary()
        .blocking_failure()
        .expect("actual late cycle violation");
    match failure.fields() {
        InvariantViolationFields::Acyclicity {
            contract_id,
            source,
            target,
            ..
        } => {
            assert_eq!(contract_id.as_str(), "no_cycles");
            assert_eq!(
                source,
                &crate::transactions::data::primitives::EntityReference::Existing(e)
            );
            assert_eq!(
                target,
                &crate::transactions::data::primitives::EntityReference::Existing(a)
            );
        }
        other => panic!("expected actual acyclicity violation, got {other:?}"),
    }
}

#[test]
fn minimum_cardinality_current_version_scans_only_live_slots() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let partition_id = PartitionId::main();
    let (mut partition, _, relations) =
        relation_partition_for_test(partition_id, 3, &[(0, 2, KindId(2)), (0, 1, KindId(2))]);
    partition
        .relation_arena
        .retire_for_request(execution, relations[0].slot_index(), VersionId(2))
        .unwrap()
        .unwrap();
    partition
        .adjacency
        .get_mut(0)
        .unwrap()
        .remove_adjacent_id_for_test(KindId(2), &relations[0]);
    partition
        .reverse_adjacency
        .get_mut(2)
        .unwrap()
        .remove_adjacent_id_for_test(KindId(2), &relations[0]);
    let instrumentation = RuntimeInstrumentation::new();
    let results = evaluate(
        cardinality_minimum_registry(),
        &BTreeMap::from([(partition_id, partition)]),
        VersionId(2),
        InvariantRequestProfile::CertificationBoundary,
        None,
        &instrumentation,
    );

    let violating_contracts = results
        .results()
        .iter()
        .filter_map(|result| match (&result.rule, &result.verdict) {
            (
                crate::validation::data::execution::InvariantReportedRule::Native(
                    crate::validation::data::rules::InvariantRule::CardinalityMinimumContract(
                        contract,
                    ),
                ),
                crate::validation::data::execution::InvariantVerdict::Violation(_),
            ) => Some(contract.contract_id.as_str()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(violating_contracts, ["min_one", "min_one_again"].into());

    let counters = instrumentation.complexity_snapshot();
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
    let partition_id = PartitionId::main();
    let (partition, _, _) =
        relation_partition_for_test(partition_id, 2, &[(0, 1, KindId(2)), (1, 0, KindId(2))]);
    let instrumentation = RuntimeInstrumentation::new();
    let results = evaluate(
        cardinality_minimum_registry(),
        &BTreeMap::from([(partition_id, partition)]),
        VersionId(1),
        InvariantRequestProfile::CertificationBoundary,
        None,
        &instrumentation,
    );
    let passing_contracts = results
        .results()
        .iter()
        .filter_map(|result| match (&result.rule, &result.verdict) {
            (
                crate::validation::data::execution::InvariantReportedRule::Native(
                    crate::validation::data::rules::InvariantRule::CardinalityMinimumContract(
                        contract,
                    ),
                ),
                crate::validation::data::execution::InvariantVerdict::Pass,
            ) => Some(contract.contract_id.as_str()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(passing_contracts, ["min_one", "min_one_again"].into());
    let counters = instrumentation.complexity_snapshot();
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
