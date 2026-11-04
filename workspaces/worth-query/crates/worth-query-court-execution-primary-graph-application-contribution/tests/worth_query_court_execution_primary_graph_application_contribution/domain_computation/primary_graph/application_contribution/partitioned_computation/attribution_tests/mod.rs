//! Which owner calls the sealed read set keeps for each fact, for an owner
//! reading through the reader of a real admitted operation.

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::StatusReadBy;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::routed;

use std::sync::Mutex;
use serde::Serialize;
use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_program::ApplicationArtifactDependency;
use worth_query_declaration::facade::application_program::ApplicationArtifactRetention;
use worth_query_declaration::facade::application_program::ApplicationArtifactSuccession;
use worth_query_declaration::facade::application_program::ApplicationComputationExecution;
use worth_query_declaration::facade::application_program::ApplicationComputationInput;
use worth_query_declaration::facade::application_program::ApplicationComputationPartition;
use worth_query_declaration::facade::application_program::ApplicationComputationReuse;
use worth_query_declaration::facade::application_program::ApplicationComputationStopped;
use worth_query_declaration::facade::application_program::ApplicationDerivedArtifact;
use worth_query_declaration::facade::application_program::ApplicationFeature;
use worth_query_declaration::facade::application_program::ApplicationFeatureInputLeaf;
use worth_query_declaration::facade::application_program::ApplicationLocalityGranule;
use worth_query_declaration::facade::application_program::ApplicationLocalityScope;
use worth_query_declaration::facade::application_program::ApplicationManagedComputation;
use worth_query_declaration::facade::application_program::ApplicationOutputPort;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial;
use worth_query_execution::facade::application_contribution::WorthQueryComputationInputDenial;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionMembers;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionPlan;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionView;
use worth_query_execution::facade::application_contribution::WorthQueryComputationReader;
use worth_query_execution::facade::application_contribution::WorthQueryDeterministicReducer;
use worth_query_execution::facade::application_contribution::WorthQueryInstalledPartitionedComputation;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationOwner;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationFactReaders;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema as Schema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationCheckpoint;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationDenial;
#[test]
fn a_fact_two_gathers_read_is_kept_for_both_partitions() {
    let (readers, partitions) = routed(StatusReadBy::ItemKeys, 1, false);

    assert_eq!(partitions.len(), 2, "odd and even items are two partitions");
    // The facts are in key order: the label, then the status.
    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by_for_test(false, [], partitions),
            ComputationFactReaders::read_by_for_test(false, [1, 2, 3], []),
        ]
    );
}
#[test]
fn a_fact_the_membership_read_is_a_membership_fact() {
    let (readers, partitions) = routed(StatusReadBy::Membership, 1, false);

    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by_for_test(false, [], partitions),
            ComputationFactReaders::read_by_for_test(true, [], []),
        ]
    );
}
#[test]
fn a_fact_the_handler_also_read_stays_the_item_keys() {
    // The handler reads its own facts again on every attempt, so it is not a
    // reader the computation keeps: the status is still the item keys' fact.
    let (readers, partitions) = routed(StatusReadBy::ItemKeys, 1, true);

    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by_for_test(false, [], partitions),
            ComputationFactReaders::read_by_for_test(false, [1, 2, 3], []),
        ]
    );
}
#[test]
fn an_attempt_that_ran_two_computations_keeps_no_computation_facts() {
    // Partition identities name the partitions of one computation only.
    assert!(routed(StatusReadBy::ItemKeys, 2, false).0.is_none());
}
