use crate::WorthServerOperationRequest;

#[derive(Clone, Debug)]
pub struct WorthServerProductOperationAuthorizationRequest<'a> {
    operation_name: &'a str,
    operation_request: &'a WorthServerOperationRequest,
}

impl<'a> WorthServerProductOperationAuthorizationRequest<'a> {
    pub(crate) fn new(
        operation_name: &'a str,
        operation_request: &'a WorthServerOperationRequest,
    ) -> Self {
        Self {
            operation_name,
            operation_request,
        }
    }

    pub fn operation_name(&self) -> &str {
        self.operation_name
    }

    pub fn operation_request(&self) -> &WorthServerOperationRequest {
        self.operation_request
    }

    pub fn application_authority_proof_identity(&self) -> Option<&str> {
        self.operation_request
            .resolved_request_context()
            .request_context()
            .authenticated_principal()
            .application_authority_proof_identity()
    }
}

pub trait WorthServerProductOperationAuthorizer: std::fmt::Debug + Send + Sync {
    fn authorize(
        &self,
        request: &WorthServerProductOperationAuthorizationRequest<'_>,
    ) -> Result<
        WorthServerProductOperationAuthorization,
        WorthServerProductOperationAuthorizationDenial,
    >;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationAuthorization {
    authority_identity: String,
    plan_digest: String,
    authority_basis: String,
    canonical_digest: String,
}

impl WorthServerProductOperationAuthorization {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use super::execution_pipeline::read_batch_accounting::string;
        string(&self.authority_identity)
            .saturating_add(string(&self.plan_digest))
            .saturating_add(string(&self.authority_basis))
            .saturating_add(string(&self.canonical_digest))
    }

    pub fn new(
        authority_identity: impl Into<String>,
        plan_digest: impl Into<String>,
        authority_basis: impl Into<String>,
    ) -> Result<Self, String> {
        let authority_identity = authority_identity.into();
        let plan_digest = plan_digest.into();
        let authority_basis = authority_basis.into();
        for (label, value) in [
            ("authority identity", authority_identity.as_str()),
            ("plan digest", plan_digest.as_str()),
            ("authority basis", authority_basis.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("product operation {label} must not be blank"));
            }
        }
        let canonical_digest = crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
            "worth-server-product-operation-authorization-v1",
        )
        .field("authority", &authority_identity)
        .field("plan", &plan_digest)
        .field("basis", &authority_basis)
        .finish();
        Ok(Self {
            authority_identity,
            plan_digest,
            authority_basis,
            canonical_digest,
        })
    }

    pub fn authority_identity(&self) -> &str {
        self.authority_identity.as_str()
    }

    pub fn plan_digest(&self) -> &str {
        self.plan_digest.as_str()
    }

    pub fn authority_basis(&self) -> &str {
        self.authority_basis.as_str()
    }

    pub fn canonical_digest(&self) -> &str {
        self.canonical_digest.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationAuthorizationDenial {
    reason_key: String,
    detail: String,
}

impl WorthServerProductOperationAuthorizationDenial {
    pub fn new(reason_key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            reason_key: reason_key.into(),
            detail: detail.into(),
        }
    }

    pub fn reason_key(&self) -> &str {
        self.reason_key.as_str()
    }

    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }
}
