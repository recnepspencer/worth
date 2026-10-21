use crate::{
    WorthServerCompatibilityFacade, WorthServerCompatibilityPreparedRequest,
    WorthServerCompatibilityRequestInput, WorthServerCompletedProductOperation,
    WorthServerCompletedProductSessionCoordination, WorthServerProductOperationInput,
    WorthServerProductSessionCreationRequest,
};
use worth_proof::TransitionOutcome;

use super::{
    declared_route::WorthServerDeclaredRoute,
    operational_route::WorthServerOperationalRoute,
    request_decoding::{
        decode_json_body, header_value, query_value, WorthServerRouteBranchTarget,
        WorthServerRouteTransportRequest,
    },
    transport_denial::{WorthServerTransportDenial, WorthServerTransportDenialCode},
};

#[derive(Clone, Debug)]
pub struct WorthServerRouteExecutionBridge {
    instance_selection: crate::WorthServerInstanceSelectionPolicy,
    instance: crate::WorthServerInstanceIdentity,
    route: WorthServerRouteBridgeTarget,
    compat_http: WorthServerCompatibilityFacade,
}

impl WorthServerRouteExecutionBridge {
    pub(crate) fn semantic(
        route: WorthServerDeclaredRoute,
        compat_http: WorthServerCompatibilityFacade,
        instance: crate::WorthServerInstanceIdentity,
        instance_selection: crate::WorthServerInstanceSelectionPolicy,
    ) -> Self {
        Self {
            instance_selection,
            instance,
            route: WorthServerRouteBridgeTarget::Semantic(route),
            compat_http,
        }
    }

    pub(crate) fn operational(
        route: WorthServerOperationalRoute,
        compat_http: WorthServerCompatibilityFacade,
        instance: crate::WorthServerInstanceIdentity,
    ) -> Self {
        Self {
            instance_selection: crate::WorthServerInstanceSelectionPolicy::Optional,
            instance,
            route: WorthServerRouteBridgeTarget::Operational(route),
            compat_http,
        }
    }

    pub fn method(&self) -> &str {
        match &self.route {
            WorthServerRouteBridgeTarget::Semantic(route) => route.method(),
            WorthServerRouteBridgeTarget::Operational(route) => route.method(),
        }
    }

    pub fn path(&self) -> &str {
        match &self.route {
            WorthServerRouteBridgeTarget::Semantic(route) => route.path(),
            WorthServerRouteBridgeTarget::Operational(route) => route.path(),
        }
    }

    pub fn operation_name(&self) -> Option<&str> {
        match &self.route {
            WorthServerRouteBridgeTarget::Semantic(route) => Some(route.operation_name()),
            WorthServerRouteBridgeTarget::Operational(_) => None,
        }
    }

    pub fn execute(
        &self,
        request: WorthServerRouteTransportRequest,
    ) -> Result<WorthServerRouteExecutionOutcome, WorthServerTransportDenial> {
        match &self.route {
            WorthServerRouteBridgeTarget::Semantic(route) => {
                if self
                    .instance_selection
                    .requires_selection(route.operation_family(), route.operation_name())
                    && header_value(&request, "x-worth-server-instance").is_none()
                {
                    return Err(WorthServerTransportDenial::new(
                        WorthServerTransportDenialCode::MissingServerInstance,
                        "this operation requires the originally selected Server instance",
                    ));
                }
                if let Some(requested) = header_value(&request, "x-worth-server-instance")
                    && self.instance.as_str() != requested
                {
                    return Err(WorthServerTransportDenial::instance_mismatch(
                        requested,
                        self.instance.as_str(),
                    ));
                }
                let prepared_request = self.prepare_request(&request, route)?;
                execute_semantic_route(&self.compat_http, route, &request, &prepared_request)
            }
            WorthServerRouteBridgeTarget::Operational(route) => {
                Ok(WorthServerRouteExecutionOutcome::Operational(
                    WorthServerOperationalRouteOutcome::new(route.kind(), route.path().to_string()),
                ))
            }
        }
    }

    fn prepare_request(
        &self,
        request: &WorthServerRouteTransportRequest,
        route: &WorthServerDeclaredRoute,
    ) -> Result<WorthServerCompatibilityPreparedRequest, WorthServerTransportDenial> {
        let mut builder = WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id(require_non_blank(
                request.authenticated_principal_id(),
                WorthServerTransportDenialCode::MissingAuthenticatedPrincipalId,
                "authenticated principal id",
            )?)
            .with_tenant_id(require_non_blank(
                request.tenant_id(),
                WorthServerTransportDenialCode::MissingTenantId,
                "tenant id",
            )?)
            .with_workspace_id(require_non_blank(
                request.workspace_id(),
                WorthServerTransportDenialCode::MissingWorkspaceId,
                "workspace id",
            )?)
            .with_route_family(route.route_family())
            .with_method(route.method())
            .with_path(route.path());
        if let Some(admitted_caller) = request.admitted_transport_caller() {
            builder = builder.with_admitted_transport_caller(admitted_caller.clone());
        }
        builder = match request.branch_target() {
            WorthServerRouteBranchTarget::Main => builder.with_main_branch(),
            WorthServerRouteBranchTarget::Branch { branch_id } => builder.with_branch_id(branch_id),
            WorthServerRouteBranchTarget::Preview { preview_id } => {
                builder.with_preview_id(preview_id)
            }
        };
        if let Some(diagnostics_profile) = request.diagnostics_profile() {
            builder = builder.with_diagnostics_profile(diagnostics_profile);
        }
        for (name, value) in request.headers() {
            builder = builder.with_header(name, value);
        }
        for (name, value) in request.query_pairs() {
            builder = builder.with_query_pair(name, value);
        }
        if let Some(content_type) = request.body_content_type() {
            builder = builder.with_body_content_type(content_type);
        }
        builder = builder.with_body_present(request.body_present());
        let input = builder.build().map_err(|error| {
            WorthServerTransportDenial::new(
                WorthServerTransportDenialCode::MissingBranchTarget,
                format!("failed to build compatibility request input: {error:?}"),
            )
        })?;
        match self.compat_http.prepare_request(input) {
            TransitionOutcome::Success(prepared) => Ok(prepared),
            TransitionOutcome::Denied(denial) => Err(WorthServerTransportDenial::new(
                WorthServerTransportDenialCode::UnknownRoute,
                denial.detail().to_string(),
            )),
            other => Err(WorthServerTransportDenial::new(
                WorthServerTransportDenialCode::UnknownRoute,
                format!("compatibility request could not be prepared: {other:?}"),
            )),
        }
    }
}

#[derive(Clone, Debug)]
enum WorthServerRouteBridgeTarget {
    Semantic(WorthServerDeclaredRoute),
    Operational(WorthServerOperationalRoute),
}

#[derive(Clone, Debug)]
pub enum WorthServerRouteExecutionOutcome {
    ProductOperation(Box<WorthServerCompletedProductOperation>),
    ProductSession(Box<WorthServerCompletedProductSessionCoordination>),
    Operational(WorthServerOperationalRouteOutcome),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationalRouteOutcome {
    kind: super::WorthServerOperationalRouteKind,
    path: String,
}

impl WorthServerOperationalRouteOutcome {
    fn new(kind: super::WorthServerOperationalRouteKind, path: String) -> Self {
        Self { kind, path }
    }

    pub fn kind(&self) -> super::WorthServerOperationalRouteKind {
        self.kind
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

fn execute_semantic_route(
    compat_http: &WorthServerCompatibilityFacade,
    route: &WorthServerDeclaredRoute,
    request: &WorthServerRouteTransportRequest,
    prepared_request: &WorthServerCompatibilityPreparedRequest,
) -> Result<WorthServerRouteExecutionOutcome, WorthServerTransportDenial> {
    let payload_json = decode_json_body(request, route.payload_schema_identity())?;
    match route.operation_family() {
        crate::WorthServerOperationFamily::ProductApplicationRead => {
            let mut input = WorthServerProductOperationInput::new(
                route.operation_name(),
                crate::WorthServerProductOperationPayload::json(
                    route.payload_schema_identity(),
                    payload_json
                        .clone()
                        .unwrap_or_else(|| serde_json::json!({})),
                ),
            );
            if let Some(product_session_identity) = header_value(request, "x-product-session-id")
                .or_else(|| query_value(request, "product_session"))
            {
                input = input.with_product_session_identity(product_session_identity);
            }
            let operation = compat_http
                .product_operations()
                .execute_product_protocol(prepared_request, input)
                .map_err(WorthServerTransportDenial::from_product_operation)?;
            Ok(WorthServerRouteExecutionOutcome::ProductOperation(
                Box::new(operation),
            ))
        }
        crate::WorthServerOperationFamily::ProductApplicationMutation => {
            let body = payload_json
                .clone()
                .unwrap_or_else(|| serde_json::json!({}));
            let mut input = WorthServerProductOperationInput::new(
                route.operation_name(),
                crate::WorthServerProductOperationPayload::json(
                    route.payload_schema_identity(),
                    body,
                ),
            );
            if let Some(product_session_identity) = header_value(request, "x-product-session-id")
                .or_else(|| query_value(request, "product_session"))
            {
                input = input.with_product_session_identity(product_session_identity);
            }
            if let Some(basis_digest) = query_value(request, "basis") {
                input = input.with_basis_digest(basis_digest);
            }
            if let Some(idempotency_key) = header_value(request, "idempotency-key") {
                input = input.with_idempotency_key(
                    crate::WorthServerProductIdempotencyKey::new(idempotency_key).map_err(
                        |detail| {
                            WorthServerTransportDenial::new(
                                WorthServerTransportDenialCode::InvalidIdempotencyKey,
                                detail,
                            )
                        },
                    )?,
                );
            }
            let operation = compat_http
                .product_operations()
                .execute_product_protocol(prepared_request, input)
                .map_err(WorthServerTransportDenial::from_product_operation)?;
            Ok(WorthServerRouteExecutionOutcome::ProductOperation(
                Box::new(operation),
            ))
        }
        crate::WorthServerOperationFamily::ProductSessionCoordination => {
            let body = payload_json.unwrap_or_else(|| serde_json::json!({}));
            let completed = match route.operation_name() {
                "product_session.open_preview" => compat_http
                    .product_sessions()
                    .open_preview_with_proof(
                        prepared_request,
                        decode_session_creation_request(route.operation_name(), &body),
                    ),
                "product_session.open_mutation" => compat_http
                    .product_sessions()
                    .open_mutation_with_proof(
                        prepared_request,
                        decode_session_creation_request(route.operation_name(), &body),
                    ),
                "product_session.close" => {
                    let product_session_identity =
                        header_value(request, "x-product-session-id")
                            .or_else(|| query_value(request, "product_session"))
                            .ok_or_else(|| {
                                WorthServerTransportDenial::new(
                                    WorthServerTransportDenialCode::MissingProductSessionIdentity,
                                    "product session close routes require a product session identity in `x-product-session-id` header or `product_session` query parameter",
                                )
                            })?;
                    compat_http.product_sessions().close_with_proof(
                        prepared_request,
                        &crate::WorthServerProductSessionIdentity::new(product_session_identity),
                    )
                }
                operation_name => {
                    return Err(WorthServerTransportDenial::new(
                        WorthServerTransportDenialCode::UnknownRoute,
                        format!(
                            "route-declared session operation `{operation_name}` is unsupported"
                        ),
                    ))
                }
            }
            .map_err(|denial| {
                WorthServerTransportDenial::new(
                    WorthServerTransportDenialCode::UnknownRoute,
                    denial.detail().to_string(),
                )
            })?;
            Ok(WorthServerRouteExecutionOutcome::ProductSession(Box::new(
                completed,
            )))
        }
        family => Err(WorthServerTransportDenial::new(
            WorthServerTransportDenialCode::UnknownRoute,
            format!(
                "route execution does not yet support operation family `{}`",
                family.as_str()
            ),
        )),
    }
}

fn decode_session_creation_request(
    operation_name: &str,
    body: &serde_json::Value,
) -> WorthServerProductSessionCreationRequest {
    let default_operation_name = match operation_name {
        "product_session.open_preview" => "product_editor.render_preview",
        "product_session.open_mutation" => "product_editor.apply",
        other => other,
    };
    let mut request = WorthServerProductSessionCreationRequest::for_operation(
        body.get("operation_name")
            .and_then(|value| value.as_str())
            .unwrap_or(default_operation_name),
    );
    if let Some(basis_digest) = body.get("basis_digest").and_then(|value| value.as_str()) {
        request = request.with_basis_digest(basis_digest);
    }
    if let Some(expiry_seconds) = body.get("expiry_seconds").and_then(|value| value.as_u64()) {
        request = request.with_expiry_seconds(expiry_seconds);
    }
    request
}

fn require_non_blank<'a>(
    value: &'a str,
    code: WorthServerTransportDenialCode,
    label: &str,
) -> Result<&'a str, WorthServerTransportDenial> {
    if value.trim().is_empty() {
        return Err(WorthServerTransportDenial::new(
            code,
            format!("route transport request requires a non-blank {label}"),
        ));
    }
    Ok(value)
}
