//! Crate-private upward boundary ports for `worth-query-execution-query`.

use std::any::Any;

use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntimeInner;
use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;

/// Internal projection from either `WorthQueryPrimaryGraphApplicationRuntimeInner`
/// or the outer `worth-query-execution` wrapper to the inner query runtime.
#[doc(hidden)]
pub trait ApplicationRuntimeRef<Schema> {
    fn runtime_inner(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>;

    fn outer_application_any(&self) -> Option<&(dyn Any + Send + Sync)> {
        None
    }
}

impl<Schema> ApplicationRuntimeRef<Schema>
    for WorthQueryPrimaryGraphApplicationRuntimeInner<Schema>
{
    #[inline]
    fn runtime_inner(&self) -> &WorthQueryPrimaryGraphApplicationRuntimeInner<Schema> {
        self
    }
}

/// Internal projection from either `WorthQuerySelectedProductOperationInner`
/// or the outer `worth-query-execution` wrapper to the inner selected product.
#[doc(hidden)]
pub trait SelectedProductOperationRef<'runtime, Schema> {
    fn selected_inner(&self) -> &WorthQuerySelectedProductOperation<'runtime, Schema>;
}

impl<'runtime, Schema> SelectedProductOperationRef<'runtime, Schema>
    for WorthQuerySelectedProductOperation<'runtime, Schema>
{
    #[inline]
    fn selected_inner(&self) -> &WorthQuerySelectedProductOperation<'runtime, Schema> {
        self
    }
}

/// Upward port implemented by `worth-query-execution`'s conditional cleanup
/// routes so `DecisionReader::prepare_conditional_definition` resolves live
/// authored or temporal conditional predecessors without retaining a strong
/// reference that could alter runtime teardown order.
#[doc(hidden)]
pub trait ConditionalDefinitionAdmissionPort: Send + Sync {
    fn resolve_predecessor(
        &self,
        bridge_root: &worth_query_execution_contracts::facade::ConditionalOwner,
        product: &crate::basis::WorthQueryProductBranchLease,
        binding_identity: &str,
        lease: &std::sync::Arc<
            crate::domain_computation::primary_graph::conditional_operation::installation::clock_handle::ConditionalClockLease,
        >,
    ) -> Result<
        Result<
            (
                std::sync::Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
                bool,
            ),
            worth_runtime_bridge::facade::BridgeConditionalDenial,
        >,
        crate::domain_computation::primary_graph::WorthQueryApplicationConditionalDefinitionAdmissionDenial,
    >;
}

/// Crate-boundary port exposing owner-private read-only inspection on
/// `WorthQueryAdmittedApplicationCapabilityAccess` strictly to `worth-query-execution`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait AdmittedCapabilityAccessInspectionPort {
    fn runtime_authority_for(
        &self,
        receiver: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        worth_proof::HomeIncarnationDenial,
    >;

    fn principal_entity_id(&self) -> worth_relational::facade::identity::EntityId;

    fn resource_entity_id(&self) -> worth_relational::facade::identity::EntityId;
}

/// Crate-boundary port exposing `WorthQueryExecutionRuntime::retain_primary_graph_integration_handle`
/// strictly to `worth-query-execution`. Never exported on `worth-query-execution::facade` or Host.
pub trait ExecutionRuntimePrimaryGraphPort {
    fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle>;
}


/// Crate-boundary port exposing owner-private `WorthQueryAdmittedApplicationQueryPlan`
/// fields strictly to `worth-query-execution`. Never exported
/// on `worth-query-execution::facade`, Publication, or Host.
pub trait AdmittedQueryPlanInternalPort<
    'a,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>
{
    fn basis(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_query::basis::WorthQueryApplicationQueryBasisCustody;

    fn controls(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryControls<'a>;

    fn governance(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance;

    fn take_governance_readmission(
        &mut self,
    ) -> Option<
        crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryPendingApplicationQueryGovernance,
    >;

    fn history_metadata(
        &self,
    ) -> Option<
        &crate::domain_computation::primary_graph::application_query::history::HistoryMetadataAdmission,
    >;

    fn set_history_metadata(
        &mut self,
        metadata: Option<
            crate::domain_computation::primary_graph::application_query::history::HistoryMetadataAdmission,
        >,
    );

    fn parameters(
        &self,
    ) -> &worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;

    fn principal_ref(
        &self,
    ) -> &'a crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        Schema,
        Principal,
        PrincipalIdentity,
    >;

    fn query(
        &self,
    ) -> &'a worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Scope,
    >;

    fn scope(
        &self,
    ) -> &'a crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
        Schema,
        Scope,
    >;

    fn security_product(&self) -> &crate::basis::WorthQueryProductObservationLease;

    fn carry_planned_batch_item(
        &mut self,
        item: Option<
            crate::domain_computation::primary_graph::application_query::batch::PlannedBatchItem,
        >,
    );
}
