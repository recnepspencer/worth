use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::model::assert_created_partition;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::model::observe;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::world::mixed_effect_world;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_binding::prepare_provider_attempt;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationRealizedEffect;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedRelation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::identity::RelationId;

#[test]
fn mixed_effects_lower_to_the_exact_independent_semantic_model() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = mixed_effect_world(execution);
    let prepared = prepare_provider_attempt(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::schema_layout::unique_fields::WorthQueryUniqueFields::none_for_test(),
        &|_, _| None,
        PartitionId::main(),
        world.effects.len(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::scope_bindings::CompletedReadScopes::EmptyOwnerFixture, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::dependency_identity::content_tree::CompletedFactRoutes::from_facts_for_test(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload::native(admitted(world.facts)), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), None, None,
        Vec::new(),
        world.effects,
        None,
        None,
        Default::default(),
        false,
        false,
        &[],
        None,
    )
    .expect("complete mixed effect basis should lower");

    assert_eq!(observe(prepared), world.expected);
}

#[test]
fn alternate_effect_insertion_preserves_each_exact_association_and_order() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = mixed_effect_world(execution);
    let prepared = prepare_provider_attempt(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::schema_layout::unique_fields::WorthQueryUniqueFields::none_for_test(),
        &|_, _| None,
        PartitionId::main(),
        world.alternate_effects.len(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::scope_bindings::CompletedReadScopes::EmptyOwnerFixture, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::dependency_identity::content_tree::CompletedFactRoutes::from_facts_for_test(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload::native(admitted(world.facts)), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), None, None,
        Vec::new(),
        world.alternate_effects,
        None,
        None,
        Default::default(),
        false,
        false,
        &[],
        None,
    )
    .expect("complete mixed effect basis should lower");
    assert_eq!(observe(prepared), world.alternate_expected);
}

#[test]
fn created_records_and_symbolic_endpoints_use_the_issued_mutation_partition() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = mixed_effect_world(execution);
    let prepared = prepare_provider_attempt(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::schema_layout::unique_fields::WorthQueryUniqueFields::none_for_test(),
        &|_, _| None,
        PartitionId(7),
        world.effects.len(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::scope_bindings::CompletedReadScopes::EmptyOwnerFixture, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::dependency_identity::content_tree::CompletedFactRoutes::from_facts_for_test(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload::native(admitted(world.facts)), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), None, None,
        Vec::new(),
        world.effects,
        None,
        None,
        Default::default(),
        false,
        false,
        &[],
        None,
    )
    .expect("complete mixed effect basis should lower into the issued partition");

    assert_created_partition(prepared, PartitionId(7));
}

#[test]
fn two_relation_deletes_from_one_adjacency_share_one_provisional_retirement() {
    let from = EntityId::new(PartitionId::main(), 1, 0);
    let first_to = EntityId::new(PartitionId::main(), 2, 0);
    let second_to = EntityId::new(PartitionId::main(), 3, 0);
    let first_relation = RelationId::new(PartitionId::main(), 4, 0);
    let second_relation = RelationId::new(PartitionId::main(), 5, 0);
    let relation_kind = KindId::new(31);
    let facts = vec![WorthQueryApplicationObservedFact::Adjacency {
        relation_kind,
        anchor: from,
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: 2,
        relations: vec![
            WorthQueryApplicationObservedRelation {
                relation_id: first_relation,
                from,
                to: first_to,
            },
            WorthQueryApplicationObservedRelation {
                relation_id: second_relation,
                from,
                to: second_to,
            },
        ],
    }];
    let effects = vec![
        WorthQueryApplicationRealizedEffect::DeleteRelation {
            relation_id: first_relation,
        },
        WorthQueryApplicationRealizedEffect::DeleteRelation {
            relation_id: second_relation,
        },
    ];
    let prepared = prepare_provider_attempt(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::schema_layout::unique_fields::WorthQueryUniqueFields::none_for_test(),
        &|_, _| None,
        PartitionId::main(),
        effects.len(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::scope_bindings::CompletedReadScopes::EmptyOwnerFixture, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::dependency_identity::content_tree::CompletedFactRoutes::from_facts_for_test(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload::native(admitted(facts)), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), None, None,
        Vec::new(),
        effects,
        None,
        None,
        Default::default(),
        false,
        false,
        &[],
        None,
    )
    .expect("both relation deletes are authorized by the observed adjacency");

    assert_eq!((*prepared.effects_for_test()).expected_steps_for_test().len(), 1);
}

pub fn admitted<T>(values: Vec<T>) -> worth_execution::ExecutionArray<T> {
    let mut array = worth_execution::ExecutionArrayBuilder::allocate(
        values.len(),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    for value in values {
        array.push(value).unwrap();
    }
    array.seal().unwrap()
}

#[path = "authoritative_truth/mod.rs"]
pub mod authoritative_truth;
#[path = "model/mod.rs"]
pub mod model;
#[path = "world/mod.rs"]
pub mod world;
