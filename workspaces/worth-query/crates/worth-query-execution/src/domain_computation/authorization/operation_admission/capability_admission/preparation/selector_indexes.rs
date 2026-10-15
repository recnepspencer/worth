//! Exact equality-index currency consumed by one admitted capability projection.
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationAuthorizationRefusalKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;

pub(super) fn prepare<Schema: ApplicationSchema, Scope, Context>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    session: &WorthQueryManagedGraphWorkSession,
    projection: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
    operation: &str,
    principal: &crate::domain_computation::primary_graph::WorthQueryPrimaryPrincipalBindingLayout,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let denied = |cause: String| {
        WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::GraphWorkAdmissionUnavailable,
            ),
            cause,
        )
    };
    let graph = application
        .runtime
        .primary_graph()
        .ok_or_else(|| denied("capability selector graph".into()))?;
    let product = session
        .mutation_product()
        .ok_or_else(|| denied("capability selected basis".into()))?;
    let basis = product.relational_basis();
    let observation = basis.observation();
    let Some(source_commit_id) = observation.commit_id() else {
        return Ok(());
    };
    let mut indexes = vec![principal.selector_index()];
    let mut select = |entity: &str, aspect: &str, field: &str| {
        if !graph
            .layout()
            .authorization_selector_is_declared(operation, entity, aspect, field)
        {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    WorthQueryOperationAuthorizationRefusalKind::CapabilityProjectionRejected,
                ),
                format!("undeclared capability selector {operation}/{entity}/{aspect}/{field}"),
            ));
        }

        let index = graph
            .layout()
            .equality_selector_index(entity, aspect, field)
            .ok_or_else(|| denied(format!("capability selector index {field}")))?;
        if !indexes.contains(&index) {
            indexes.push(index);
        }
        Ok::<_, WorthQueryOperationAuthorizationDenial>(())
    };
    let resource = projection.resource();
    select(resource.entity(), resource.aspect(), resource.field())?;
    if let Some(selector) = projection.elevation_selector() {
        select(selector.entity(), selector.aspect(), selector.field())?;
    }
    if let Some(related) = projection.related() {
        let selector = related.selector();
        select(selector.entity(), selector.aspect(), selector.field())?;
    }
    for selected in projection.context_value().entities() {
        let selector = selected.selector();
        select(selector.entity(), selector.aspect(), selector.field())?;
    }
    let handle = graph.integration_handle();
    handle.with_runtime_mut(execution, |runtime| {
        runtime.index_authority().refresh_field_indexes_for_basis(
            DerivedIndexBuildRequest { source_commit_id, branch_id: observation.identity().branch_id().clone(), index_ids: indexes },
            basis, None, crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget(), execution,
        ).map(|_| ()).map_err(|cause| denied(format!("capability selector index preparation: {cause:?}")))
    })
}
