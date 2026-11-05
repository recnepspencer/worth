//! Graph continuation of the exact permission prepared before a Clean probe.

use crate::domain_computation::primary_graph::application_query::admission::finish::admitted_graph::AdmittedGraphParentOps;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::*;
use crate::domain_computation::primary_graph::application_query::admission::permission::PreparedApplicationQueryPermission;
use crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance;

use crate::domain_computation::provider_session::WorthQueryPreparedProductReadSessionIdentity;

pub(in crate::domain_computation::primary_graph) trait PreparedPermissionScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn finish_prepared_application_query_permission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        permission: PreparedApplicationQueryPermission<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        self.finish_prepared_application_query_permission_with_native_access(execution, permission, &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }

    fn finish_prepared_application_query_permission_with_native_access<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        permission: PreparedApplicationQueryPermission<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let PreparedApplicationQueryPermission {
            issuer,
            query,
            access,
            parameters,
            controls,
            basis,
            security_basis,
            security_product,
            session,
            disclosure,
            authorization,
            authorization_work,
            selected_access,
            shape,
        } = permission;

        if !std::ptr::addr_eq(self, issuer) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                query.name(),
            ));
        }
        let graph = self.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            )
        })?;
        let prepared = self.prepare_application_query_graph_work_admitted(
            query,
            &parameters,
            &controls,
            shape,
            graph,
            disclosure,
        )?;
        let indexes = crate::domain_computation::primary_graph::application_query::basis::ensure_query_truth_and_security_indexes_admitted_with_native_access(execution, graph, query, basis.selected_product().relational_basis(), security_product.relational_basis(), native)?;
        let index_posture = WorthQueryQueryIndexPosture::SelectedInstalled(indexes);
        validate_admission_request(controls.request_scope(), query.name())?;
        let graph_work = start_prepared_query_session(
            self.application_query_host(),
            session,
            prepared.plan,
            &prepared.obligation_identity,
            query,
            &access,
            &basis,
            &security_product,
            graph,
        )?;
        if !authorization.belongs_to_session(graph_work.identity()) {
            return Err(graph_work_denial(query.name()));
        }
        let mut graph_work = graph_work;
        graph_work.set_retained_decision_facts(authorization.exact_fact_count());
        Ok(WorthQueryAdmittedApplicationQueryPlan {
            history_metadata: None,
            planned_batch_item: None,
            retain_expression_context: false,
            retain_projection_for_inspection: false,
            runtime_authority: self.authority_identity(),
            graph_authority_identity: self
                .primary_graph_authority()
                .authority_identity()
                .to_string(),
            provider_identity: self
                .primary_graph_authority()
                .provider_identity()
                .to_string(),
            query,
            principal: access.principal(),
            scope: access.scope(),
            parameters,
            controls,
            canonical_work: prepared.canonical_work,
            continuation_index_id: prepared.continuation_index_id,
            continuation_state: None,
            basis,
            security_basis,
            index_posture,
            security_product,
            graph_work,
            authorization,
            authorization_work,
            selected_access: Some(selected_access),
            governance: WorthQueryApplicationQueryGovernance::Public,
        })
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > PreparedPermissionScopedOps<Schema> for HostRuntime
{
}

#[allow(clippy::too_many_arguments)]
fn start_prepared_query_session<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
    reserved: WorthQueryPreparedProductReadSessionIdentity,
    plan: WorthQueryAdmittedGraphWorkPlan,
    obligation: &WorthQueryInstalledGraphObligationSetIdentity,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    access: &WorthQueryApplicationQueryAccessContext<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    basis: &WorthQueryApplicationQueryBasisCustody,
    security_product: &crate::basis::WorthQueryProductObservationLease,
    graph: &WorthQueryPrimaryGraph,
) -> Result<WorthQueryManagedGraphWorkSession, WorthQueryApplicationQueryAdmissionDenial>
where
    Schema: ApplicationSchema,
{
    WorthQueryManagedGraphWorkSession::start_query_with_reserved_identity(
        reserved,
        plan,
        application.authority_identity(),
        query.binding_identity(),
        obligation,
        query.authority_identity(),
        access.principal().principal_entity_id(),
        WorthQueryGraphWorkAccessContextAffinity::entity(access.scope().entity_id()),
        basis.identity(),
        basis.retained_product(),
        security_product,
        application
            .application_runtime()
            .graph_work_provider_identity(),
        graph.product_read_session_port(),
    )
    .map_err(|cause| graph_work_denial(format!("{}: {cause:?}", query.name())))
}
