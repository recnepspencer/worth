//! Auto-observed unique child identities use the installed equality index.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::unique_write_dependencies::UNIQUE_VALUE_CANDIDATE_LIMIT;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationRealizedEffect;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection::observe_indexed_entity_selection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::unique_write_dependencies::observe_unique_values;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::merge_unique_values::MergeWorld;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_relational::facade::history::BranchId;
#[test]
fn delegation_child_identity_records_absence_and_taken_values_once() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = MergeWorld::new();
    let held = world.create(execution, "main", "held-child", 7);
    let unique = world.unique(Some(world.index));
    world.mutate(execution, |runtime| {
        let identity = runtime.branch_identity(&BranchId("main".to_owned())).unwrap();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let snapshot = runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap();
        for (ordinal, holders) in [(8, Vec::new()), (7, vec![held])] {
            let value = AspectValue::String(InternedString::from(format!("value-{ordinal}")));
            let effect = || WorthQueryApplicationRealizedEffect::CreateEntity {
                kind: world.kind, key: "delegated-child".to_owned(),
                fields: std::collections::BTreeMap::from([(world.label.clone(), value.clone())]),
                partition: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationCreationPartition::Issued,
            };
            let mut calls = 0;
            let facts = observe_unique_values(&[], unique.fields(), &[effect(), effect()], |_| None, |index, kind, locator, value| {
                calls += 1;
                observe_indexed_entity_selection(runtime,execution,  &snapshot, index, kind, locator, value, UNIQUE_VALUE_CANDIDATE_LIMIT)
            }).unwrap();
            assert_eq!(calls, 1, "one distinct created identity has one lookup");
            assert_eq!(facts.len(), 1);
            let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } = &facts[0] else { panic!("the exact indexed selection is carried") };
            assert_eq!(candidates, &holders);
            let lower = |facts: Vec<WorthQueryApplicationObservedFact>, effects: Vec<WorthQueryApplicationRealizedEffect>| {
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_binding::prepare_provider_attempt(
                    unique.fields(), &|_, _| None,
                    worth_relational::facade::identity::PartitionId::main(),
                    effects.len(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::scope_bindings::CompletedReadScopes::EmptyOwnerFixture, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::dependency_identity::content_tree::CompletedFactRoutes::from_facts_for_test(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload::native(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::admitted_arrays::admit_array(facts.len(), facts, worth_execution::ExecutionAllocationPolicy::SystemAllocation, "unique fixture", || Ok(())).unwrap()), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), None, None, Vec::new(), effects, None, None,
                    Default::default(), false, false, &[], None,
                ).map(|_| ()).map_err(|denial| denial.kind())
            };
            assert_eq!(lower(facts.clone(), vec![effect()]),
                if holders.is_empty() { Ok(()) } else { Err(WorthQueryApplicationAttemptDenialKind::UniqueValueTaken) });
            if holders.is_empty() {
                let mut other = effect();
                let WorthQueryApplicationRealizedEffect::CreateEntity { key, .. } = &mut other else { unreachable!() };
                *key = "delegated-other-child".to_owned();
                assert_eq!(lower(facts.clone(), vec![effect(), other]),
                    Err(WorthQueryApplicationAttemptDenialKind::UniqueValueTaken),
                    "one delegation cannot write its unique child identity twice");
            }

            assert!(observe_unique_values(&facts, unique.fields(), &[effect()], |_| None, |_, _, _, _| panic!("already observed identity is not read again")).unwrap().is_empty());
        }
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
