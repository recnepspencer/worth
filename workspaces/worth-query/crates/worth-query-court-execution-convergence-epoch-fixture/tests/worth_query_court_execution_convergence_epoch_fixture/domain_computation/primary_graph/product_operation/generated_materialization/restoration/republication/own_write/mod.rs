//! Native suspension/restoration must preserve the computation's original basis.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::generated_materialization::ProducerQualification;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_generated_own_write;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::own_write_computation::OwnWriteOutputs;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::ReadRetainedAccount;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::RetainedFamily;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountMembershipTag;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryCurrentOutputDenialKind;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_relational::facade::branch::RelationalEntityMaterialization;
use worth_relational::facade::transactions::AspectFieldPatch;

#[test]
fn republication_cannot_certify_output_computed_before_its_own_write() {
    suspend_restore_and_read();
}
#[path = "suspend_restore/mod.rs"]
pub mod suspend_restore;
use suspend_restore::suspend_restore_and_read;
