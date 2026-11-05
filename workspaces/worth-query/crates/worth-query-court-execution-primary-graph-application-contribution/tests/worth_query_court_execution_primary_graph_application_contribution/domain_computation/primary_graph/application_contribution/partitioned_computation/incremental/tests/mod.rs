//! A producer's next run of a partitioned computation, through a real
//! admitted operation. The test hands each attempt the prior a producer would
//! and keeps the run seal retains; a moved fact is the retained fact standing
//! in for an older one.

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::Attempt;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::Installed;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::Outcome;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::Owner;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::Reader;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::Root;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::StatusRead;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::TestOwner;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::attempt;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::attempt_in;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::edition;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::first_run;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::installed;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::installed_over;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::prior_of;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::sum;

pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation as QueryObservation;
pub use std::sync::Arc;
pub use std::sync::Mutex;
pub use worth_execution::PartitionItemId;
pub use worth_foundational::facade::ExecutionReport;
pub use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;
pub use worth_query_declaration::facade::application_program::ApplicationManagedComputation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Computation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Feature;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Input;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Number;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Parity;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::installed::ComputationRetention;
pub use worth_query_execution::facade::application_contribution::WorthQueryComputationInputDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionMembers;
pub use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionPlan;
pub use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionView;
pub use worth_query_execution::facade::application_contribution::WorthQueryComputationReader;
pub use worth_query_execution::facade::application_contribution::WorthQueryDeterministicReducer;
pub use worth_query_execution::facade::application_contribution::WorthQueryInstalledPartitionedComputation;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationOwner;
pub use worth_query_execution::facade::domain_computation::partitioned_computation_runs_on_this_thread_for_test as runs;
pub use worth_query_execution::facade::domain_computation::partitioned_computation_tree_work_on_this_thread_for_test as tree_runs;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::carriage::ComputationPrior;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationFullCause as Cause;
pub use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationRunForTest as Run;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
pub use worth_query_execution::court_doors_for_test::domain_computation::request_execution::QueryRequestExecution;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema as Schema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationCheckpoint;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationExecution;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::projected_attempt::attempt_placed;

#[path = "advancement_charge/mod.rs"]
pub mod advancement_charge;
#[path = "call_sequence_memory/mod.rs"]
pub mod call_sequence_memory;
#[path = "carrying/mod.rs"]
pub mod carrying;
#[path = "certified/mod.rs"]
pub mod certified;
#[path = "collision/mod.rs"]
pub mod collision;
#[path = "comparison_read_stop/mod.rs"]
pub mod comparison_read_stop;
#[path = "digests/mod.rs"]
pub mod digests;
#[path = "installation/mod.rs"]
pub mod installation;
#[path = "interruption/mod.rs"]
pub mod interruption;
#[path = "kernel_receipt/mod.rs"]
pub mod kernel_receipt;
#[path = "metadata_memory/mod.rs"]
pub mod metadata_memory;
#[path = "request_memory/mod.rs"]
pub mod request_memory;
#[path = "reroute/mod.rs"]
pub mod reroute;
#[path = "structural_walk/mod.rs"]
pub mod structural_walk;
#[path = "tree_work/mod.rs"]
pub mod tree_work;
#[path = "unobservable/mod.rs"]
pub mod unobservable;
#[path = "wide/mod.rs"]
pub mod wide;
#[path = "worker_only/mod.rs"]
pub mod worker_only;
