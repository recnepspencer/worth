pub use std::collections::BTreeMap;
pub use std::time::Duration;

pub use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
pub use worth_query_declaration::facade::application_query::ApplicationDerivedViewLimits;
pub use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_query_installation::facade::ApplicationScalarValueBinding;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::current_controls;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_ordered_query;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_query;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::markers::AccountSummaryQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::markers::OrderedAccountSummaryQuery;
pub use worth_query_execution::facade::domain_computation::WorthQueryProductBranchLease;
pub use worth_query_execution::domain_computation::primary_graph::application_query::derived_view::publication::ViewChange;
pub use worth_query_execution::domain_computation::primary_graph::application_query::derived_view::publication::ViewPublicationBasis;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedValue;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedDerivedViewDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub struct SceneLabel(String);

impl WorthQueryManagedDerivedValue for SceneLabel {
    fn retained_bytes(&self) -> usize {
        self.0.capacity()
    }
}

pub fn publication_basis(product: &WorthQueryProductBranchLease) -> ViewPublicationBasis<'_> {
    ViewPublicationBasis {
        before: product.selected_commit(),
        relational_branch: product.relational_basis().identity().branch_id(),
        product_branch: product.branch_identity(),
        incarnation: product.observation().lifecycle_incarnation(),
    }
}

pub use worth_query_execution::facade::WorthQueryDerivedPairReadPlans;

#[path = "branch_reconstruction_parity/mod.rs"]
pub mod branch_reconstruction_parity;
#[path = "dependent_reconstruction/mod.rs"]
pub mod dependent_reconstruction;
#[path = "leased_reconstruction/mod.rs"]
pub mod leased_reconstruction;
#[path = "member_token_reconciliation/mod.rs"]
pub mod member_token_reconciliation;
#[path = "membership_reconciliation/mod.rs"]
pub mod membership_reconciliation;
#[path = "native_collection/mod.rs"]
pub mod native_collection;
#[path = "optional_owner/mod.rs"]
pub mod optional_owner;
#[path = "owner_sized/mod.rs"]
pub mod owner_sized;
#[path = "reconstruction_parity/mod.rs"]
pub mod reconstruction_parity;
#[path = "reconstruction_population/mod.rs"]
pub mod reconstruction_population;
#[path = "reconstruction_stops/mod.rs"]
pub mod reconstruction_stops;
#[path = "registration_lifecycle/mod.rs"]
pub mod registration_lifecycle;
#[path = "request_refusals/mod.rs"]
pub mod request_refusals;
#[path = "scoped_query/mod.rs"]
pub mod scoped_query;
