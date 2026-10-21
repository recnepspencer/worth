#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerTransportDenialCode {
    CallerAdmissionDenied,
    MissingAuthenticatedPrincipalId,
    MissingTenantId,
    MissingWorkspaceId,
    UnsupportedContentType,
    OversizedBody,
    MalformedJson,
    InvalidIdempotencyKey,
    MissingProductSessionIdentity,
    MissingBranchTarget,
    RouteExecutionFailed,
    ProductOperationDenied,
    ServerInstanceMismatch,
    MissingServerInstance,
    UnknownRoute,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerTransportDenial {
    instance_mismatch: Option<Box<ServerInstanceMismatch>>,
    code: WorthServerTransportDenialCode,
    reason_key: Option<String>,
    detail: String,
    product_operation_denial: Option<Box<crate::WorthServerProductOperationSurfaceDenial>>,
}

impl WorthServerTransportDenial {
    pub(crate) fn new(code: WorthServerTransportDenialCode, detail: impl Into<String>) -> Self {
        Self {
            instance_mismatch: None,
            code,
            reason_key: None,
            detail: detail.into(),
            product_operation_denial: None,
        }
    }

    pub(crate) fn from_product_operation(
        denial: crate::WorthServerProductOperationSurfaceDenial,
    ) -> Self {
        Self {
            instance_mismatch: None,
            code: WorthServerTransportDenialCode::ProductOperationDenied,
            reason_key: None,
            detail: denial.detail().to_owned(),
            product_operation_denial: Some(Box::new(denial)),
        }
    }

    pub(crate) fn instance_mismatch(requested: &str, actual: &str) -> Self {
        Self {
            code: WorthServerTransportDenialCode::ServerInstanceMismatch,
            reason_key: Some("worth.server_instance_mismatch".into()),
            detail: "the selected Server instance is not this live assembly".into(),
            product_operation_denial: None,
            instance_mismatch: Some(Box::new(ServerInstanceMismatch {
                requested: requested.into(),
                actual: actual.into(),
            })),
        }
    }

    pub fn requested_server_instance(&self) -> Option<&str> {
        self.instance_mismatch
            .as_deref()
            .map(|mismatch| mismatch.requested.as_str())
    }

    pub fn actual_server_instance(&self) -> Option<&str> {
        self.instance_mismatch
            .as_deref()
            .map(|mismatch| mismatch.actual.as_str())
    }

    /// The original surface refusal, including currentness, execution boundary and recovery facts.
    pub fn product_operation_denial(
        &self,
    ) -> Option<&crate::WorthServerProductOperationSurfaceDenial> {
        self.product_operation_denial.as_deref()
    }

    pub(crate) fn with_reason_key(mut self, reason_key: impl Into<String>) -> Self {
        self.reason_key = Some(reason_key.into());
        self
    }

    pub fn code(&self) -> WorthServerTransportDenialCode {
        self.code.clone()
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn reason_key(&self) -> Option<&str> {
        self.reason_key.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ServerInstanceMismatch {
    requested: String,
    actual: String,
}
