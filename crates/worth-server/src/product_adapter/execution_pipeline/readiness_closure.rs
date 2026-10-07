use crate::{
    WorthServerOperationPreconditionPosture, WorthServerOperationQuerySupportContext,
    WorthServerOperationReadinessFacade, WorthServerOperationRegistry,
    WorthServerPreparedQueryHandoffKind, WorthServerProductBasisPrecondition,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationSurfaceDenial, WorthServerProductOperationSurfaceDenialCode,
    WorthServerQueryHandoffConfig, WorthServerQueryHandoffOperation,
    WorthServerQueryWorkspaceBindingRequest,
};

pub(in crate::product_adapter) fn close_product_operation_readiness(
    operation_registry: &WorthServerOperationRegistry,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    admission: &crate::WorthServerOperationAdmissionPosture,
    declaration: &WorthServerProductOperationDeclaration,
    resolved_request_context: &crate::WorthServerResolvedRequestContext,
) -> Result<crate::WorthServerOperationReadinessClosure, WorthServerProductOperationSurfaceDenial> {
    let readiness =
        WorthServerOperationReadinessFacade::with_operation_registry(operation_registry.clone());
    match declaration.basis_kind() {
        WorthServerProductOperationBasisKind::QueryDerived => {
            close_query_derived_product_readiness(
                &readiness,
                query_handoff_config,
                admission,
                declaration,
                resolved_request_context,
            )
        }
        WorthServerProductOperationBasisKind::PrimaryGraphApplication => {
            close_primary_graph_application_readiness(&readiness, admission, declaration)
        }
        _ => readiness
            .close_readiness(admission, None, None)
            .map_err(WorthServerProductOperationSurfaceDenial::from_readiness_denial),
    }
}

fn close_primary_graph_application_readiness(
    readiness: &WorthServerOperationReadinessFacade,
    admission: &crate::WorthServerOperationAdmissionPosture,
    declaration: &WorthServerProductOperationDeclaration,
) -> Result<crate::WorthServerOperationReadinessClosure, WorthServerProductOperationSurfaceDenial> {
    let provider = declaration
        .query_application_readiness_provider()
        .ok_or_else(|| {
            WorthServerProductOperationSurfaceDenial::new(
                WorthServerProductOperationSurfaceDenialCode::ReadinessDenied,
                "primary-graph application readiness provider was not retained after certification"
                    .to_string(),
            )
        })?;
    let snapshot = provider.inspect_application_readiness().map_err(|denial| {
        WorthServerProductOperationSurfaceDenial::new(
            WorthServerProductOperationSurfaceDenialCode::ReadinessDenied,
            format!(
                "{} denied application readiness: {}",
                provider.provider_name(),
                denial.subject()
            ),
        )
    })?;
    let prepared_kind = admission
        .authorization_proof()
        .admission()
        .query_handoff_intent()
        .kind();
    let operation = query_binding_operation(
        prepared_kind,
        declaration.operation_family(),
        declaration.operation_name(),
    );
    let query_context = WorthServerOperationQuerySupportContext::for_primary_graph_application(
        prepared_kind,
        &operation,
        &snapshot,
    );
    let precondition_posture = WorthServerProductBasisPrecondition::evaluate(
        declaration.operation_name(),
        admission.operation_request().identity().basis_digest(),
        snapshot.basis_token(),
    )
    .map(WorthServerOperationPreconditionPosture::ProductBasis)
    .map_err(WorthServerProductOperationSurfaceDenial::from_readiness_denial)?;
    readiness
        .close_readiness(admission, Some(query_context), Some(precondition_posture))
        .map_err(WorthServerProductOperationSurfaceDenial::from_readiness_denial)
}

fn close_query_derived_product_readiness(
    readiness: &WorthServerOperationReadinessFacade,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    admission: &crate::WorthServerOperationAdmissionPosture,
    declaration: &WorthServerProductOperationDeclaration,
    resolved_request_context: &crate::WorthServerResolvedRequestContext,
) -> Result<crate::WorthServerOperationReadinessClosure, WorthServerProductOperationSurfaceDenial> {
    let prepared_kind = admission
        .authorization_proof()
        .admission()
        .query_handoff_intent()
        .kind();
    let operation = query_binding_operation(
        prepared_kind,
        declaration.operation_family(),
        declaration.operation_name(),
    );
    let binding_request = WorthServerQueryWorkspaceBindingRequest::for_query_handoff(
        resolved_request_context.clone(),
        operation.clone(),
    );
    let workspace = query_handoff_config
        .workspace_provider()
        .bind_workspace(&binding_request)
        .map_err(|error| {
            WorthServerProductOperationSurfaceDenial::new(
                WorthServerProductOperationSurfaceDenialCode::ReadinessDenied,
                format!("{}: {}", error.stage(), error.message()),
            )
        })?;
    let downstream_delivery_contract = workspace.public_downstream_delivery_contract();
    let query_context = WorthServerOperationQuerySupportContext::new(
        prepared_kind,
        &operation,
        &workspace,
        &downstream_delivery_contract,
    );
    let snapshot_identity = workspace.snapshot_identity().map_err(|denial| {
        WorthServerProductOperationSurfaceDenial::new(
            WorthServerProductOperationSurfaceDenialCode::ReadinessDenied,
            format!("workspace snapshot identity: {denial}"),
        )
    })?;
    let precondition_posture = WorthServerProductBasisPrecondition::evaluate(
        declaration.operation_name(),
        admission.operation_request().identity().basis_digest(),
        &snapshot_identity.terminal_projection_for_reporting(),
    )
    .map(WorthServerOperationPreconditionPosture::ProductBasis)
    .map_err(WorthServerProductOperationSurfaceDenial::from_readiness_denial)?;
    readiness
        .close_readiness(admission, Some(query_context), Some(precondition_posture))
        .map_err(WorthServerProductOperationSurfaceDenial::from_readiness_denial)
}

fn query_binding_operation(
    prepared_kind: WorthServerPreparedQueryHandoffKind,
    operation_family: crate::WorthServerOperationFamily,
    operation_name: &str,
) -> WorthServerQueryHandoffOperation {
    match prepared_kind {
        WorthServerPreparedQueryHandoffKind::QueryRead => {
            WorthServerQueryHandoffOperation::query_read(operation_name)
        }
        WorthServerPreparedQueryHandoffKind::QueryMutation => {
            WorthServerQueryHandoffOperation::query_mutation(operation_name)
        }
        WorthServerPreparedQueryHandoffKind::WorthNativeSession
            if operation_family
                == crate::WorthServerOperationFamily::ProductApplicationMutation =>
        {
            WorthServerQueryHandoffOperation::direct_mutation(operation_name)
        }
        WorthServerPreparedQueryHandoffKind::WorthNativeSession => {
            WorthServerQueryHandoffOperation::direct_read(operation_name)
        }
    }
}
