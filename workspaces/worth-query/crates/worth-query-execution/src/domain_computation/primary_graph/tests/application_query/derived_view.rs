use std::collections::BTreeMap;
use std::time::Duration;

use worth_query_declaration::facade::application_query::{
    ApplicationDerivedViewDefinition, ApplicationDerivedViewLimits, ApplicationQueryParameterSet,
};
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::{
    current_controls, installed_authorization_world, installed_ordered_query, installed_query,
    live_scope, status_parameter, AccountStatus, AccountSummaryQuery, OrderedAccountSummaryQuery,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::{
    ViewChange, ViewPublicationBasis,
};
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedViewDenial, WorthQueryPrincipalResolutionMode,
};

mod member_token_reconciliation;
mod membership_reconciliation;
mod native_collection;
mod scoped_query;
struct SceneLabel(String);

impl WorthQueryManagedDerivedValue for SceneLabel {
    fn retained_bytes(&self) -> usize {
        self.0.capacity()
    }
}

fn publication_basis(product: &WorthQueryProductBranchLease) -> ViewPublicationBasis<'_> {
    ViewPublicationBasis {
        before: product.selected_commit(),
        relational_branch: product.relational_basis().identity().branch_id(),
        product_branch: product.branch_identity(),
        incarnation: product.observation().lifecycle_incarnation(),
    }
}

mod reconstruction_request;
mod registration_lifecycle;
use crate::domain_computation::primary_graph::WorthQueryDerivedPairReadPlans;
use reconstruction_request::serial_request;

mod leased_reconstruction;

mod reconstruction_population;

mod request_refusals;
