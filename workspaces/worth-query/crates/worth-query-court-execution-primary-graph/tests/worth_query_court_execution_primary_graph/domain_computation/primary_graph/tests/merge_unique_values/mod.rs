//! A merge writes unique values under the program law: each value it writes
//! is looked up at the target head and must be free or held only by the
//! entity the merge writes, and one merge writes each value at most once.

pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::merge_unique_values::FEATURE;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::merge_unique_values::MergeWorld;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::merge_unique_values::prepared_merge;

pub use std::collections::BTreeMap;
pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;
pub use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
pub use worth_relational::facade::history::BranchId;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::indexes::DerivedIndexId;
pub use worth_relational::facade::merge::MergeExecutionRequest;
pub use worth_relational::facade::merge::MergeIntent;
pub use worth_relational::facade::merge::PreparedMergeExecution;
pub use worth_relational::facade::runtime::RelationalRuntime;
pub use worth_relational::facade::symbols::ClientKey;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::CreateIntent;
pub use worth_relational::facade::transactions::CreatedEntityRef;
pub use worth_relational::facade::transactions::EntitySpec;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::IdentityWorld;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::merge_unique_values::admit_merge_unique_values;
pub use worth_query_execution::facade::integration::WorthQueryMergeUniqueValueDenialKind as Kind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::schema_layout::unique_fields::WorthQueryUniqueFieldFixture;
pub use worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle;
#[test]
fn a_merge_that_creates_a_value_the_target_holds_is_denied() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = MergeWorld::new();
    world.fork();
    world.create(execution, "main", "ada", 7);
    world.create(execution, FEATURE, "bob", 7);
    assert_eq!(world.merge(execution,), Err(Kind::ValueTaken));
}
#[test]
fn a_merge_of_free_values_commits() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = MergeWorld::new();
    world.fork();
    world.create(execution, "main", "ada", 7);
    world.create(execution, FEATURE, "bob", 8);
    world.create(execution, FEATURE, "cy", 9);
    assert_eq!(world.merge(execution,), Ok(()));
}
#[test]
fn one_merge_writes_each_unique_value_at_most_once() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = MergeWorld::new();
    world.fork();
    world.create(execution, FEATURE, "ada", 7);
    world.create(execution, FEATURE, "bob", 7);
    assert_eq!(world.merge(execution,), Err(Kind::ValueTaken));
}
#[test]
fn an_uninstalled_unique_index_is_its_own_denial() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = MergeWorld::new();
    world.fork();
    world.create(execution, FEATURE, "ada", 7);
    let unindexed = world.unique(None);
    let admitted = world.mutate(execution, |runtime| {
        let prepared = prepared_merge(execution, runtime);
        admit_merge_unique_values(unindexed.fields(), runtime, execution, &prepared)
            .map_err(|denial| denial.kind())
    });
    assert_eq!(admitted, Err(Kind::IndexUnavailable));
}
#[test]
fn a_value_held_past_the_candidate_limit_reads_as_taken() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = MergeWorld::new();
    for key in ["ada", "ada-restored", "ada-restored-again"] {
        world.create(execution, "main", key, 7);
    }
    world.fork();
    world.create(execution, FEATURE, "bob", 7);
    assert_eq!(world.merge(execution,), Err(Kind::ValueTaken));
}
#[test]
fn restored_duplicates_deny_merges_of_their_value_only() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = MergeWorld::new();
    world.create(execution, "main", "ada", 7);
    world.create(execution, "main", "ada-restored", 7);
    world.fork();
    world.create(execution, FEATURE, "bob", 8);
    assert_eq!(world.merge(execution,), Ok(()));
    world.create(execution, FEATURE, "cy", 7);
    assert_eq!(world.merge(execution,), Err(Kind::ValueTaken));
}

#[path = "counts/mod.rs"]
pub mod counts;
