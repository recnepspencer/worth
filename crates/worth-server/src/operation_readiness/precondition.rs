use crate::WorthServerCompatibilityPreparedRequest;

use super::{
    WorthServerOperationReadinessDenial, WorthServerOperationReadinessDenialCode,
    WorthServerOperationReadinessDenialFacts,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerOperationPreconditionPosture {
    NotRequired { canonical_digest: String },
    CompatibilityMutation(WorthServerCompatibilityMutationPrecondition),
    ProductBasis(WorthServerProductBasisPrecondition),
}

impl WorthServerOperationPreconditionPosture {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::{
            option_string, string,
        };
        match self {
            Self::NotRequired { canonical_digest } => string(canonical_digest),
            Self::ProductBasis(value) => option_string(&value.requested_basis_digest)
                .saturating_add(string(&value.observed_basis_digest))
                .saturating_add(string(&value.operation_name))
                .saturating_add(string(&value.canonical_digest)),
            Self::CompatibilityMutation(_) => u64::MAX,
        }
    }
    pub fn canonical_digest(&self) -> &str {
        match self {
            Self::NotRequired { canonical_digest } => canonical_digest,
            Self::CompatibilityMutation(precondition) => precondition.canonical_digest(),
            Self::ProductBasis(precondition) => precondition.canonical_digest(),
        }
    }

    pub(crate) fn not_required(label: &str) -> Self {
        Self::NotRequired {
            canonical_digest: format!(
                "worth-server-operation-precondition-posture-v1|class=not-required|label={label}"
            ),
        }
    }

    pub fn compatibility_mutation(&self) -> Option<&WorthServerCompatibilityMutationPrecondition> {
        match self {
            Self::CompatibilityMutation(precondition) => Some(precondition),
            Self::NotRequired { .. } | Self::ProductBasis(_) => None,
        }
    }

    pub fn product_basis(&self) -> Option<&WorthServerProductBasisPrecondition> {
        match self {
            Self::ProductBasis(precondition) => Some(precondition),
            Self::NotRequired { .. } | Self::CompatibilityMutation(_) => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductBasisPrecondition {
    requested_basis_digest: Option<String>,
    observed_basis_digest: String,
    operation_name: String,
    canonical_digest: String,
}

impl WorthServerProductBasisPrecondition {
    pub(crate) fn evaluate(
        operation_name: &str,
        requested_basis_digest: Option<&str>,
        observed_basis_digest: &str,
    ) -> Result<Self, WorthServerOperationReadinessDenial> {
        let observed_basis_digest =
            crate::WorthServerProductOperationBaseDigest::canonicalize_text(
                observed_basis_digest.to_string(),
            )
            .map_err(|error| {
                WorthServerOperationReadinessDenial::new(
                    WorthServerOperationReadinessDenialCode::InvalidPreconditionInput,
                    error,
                )
            })?;
        if let Some(expected_basis_digest) = requested_basis_digest {
            if expected_basis_digest != observed_basis_digest {
                return Err(WorthServerOperationReadinessDenial::new(
                    WorthServerOperationReadinessDenialCode::PreconditionFailed,
                    format!(
                        "product basis precondition `{expected_basis_digest}` did not match the observed owner basis `{observed_basis_digest}`"
                    ),
                )
                .with_facts(
                    WorthServerOperationReadinessDenialFacts::default()
                        .with_basis_mismatch(expected_basis_digest, observed_basis_digest),
                ));
            }
        }
        let canonical_digest = format!(
            "worth-server-product-basis-precondition-v1|operation={}|requested_basis={}|observed_basis={}",
            operation_name.trim(),
            requested_basis_digest.unwrap_or("none"),
            observed_basis_digest,
        );
        Ok(Self {
            requested_basis_digest: requested_basis_digest.map(str::to_string),
            observed_basis_digest,
            operation_name: operation_name.to_string(),
            canonical_digest,
        })
    }

    pub fn requested_basis_digest(&self) -> Option<&str> {
        self.requested_basis_digest.as_deref()
    }

    pub fn observed_basis_digest(&self) -> &str {
        &self.observed_basis_digest
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerCompatibilityMutationPrecondition {
    requested_basis_digest: Option<String>,
    if_match: Option<String>,
    observed_basis_digest: String,
    validator: String,
    request_identity_digest: String,
    canonical_digest: String,
}

impl WorthServerCompatibilityMutationPrecondition {
    pub(crate) fn evaluate(
        prepared_request: &WorthServerCompatibilityPreparedRequest,
        operation_name: &str,
        mutation_request_digest: &str,
        observed_basis_digest: &str,
        observed_product_session_identity: Option<&str>,
    ) -> Result<Self, WorthServerOperationReadinessDenial> {
        let requested_basis_digest = read_single_query_pair(prepared_request, "basis")?;
        let requested_base_branch = read_single_query_pair(prepared_request, "base-branch")?;
        let if_match = read_single_header(prepared_request, "if-match")?;
        let expected_product_session = read_single_header(prepared_request, "x-product-session")?;
        let idempotency_binding = read_single_header(prepared_request, "x-idempotency-binding")?;
        validate_requested_basis_digest(requested_basis_digest.as_deref())?;
        validate_requested_base_branch(prepared_request, requested_base_branch.as_deref())?;
        validate_idempotency_binding_inputs(prepared_request, idempotency_binding.as_deref())?;
        let validator = format!(
            "\"compat-http-mutation-validator-v1|basis:{}|operation:{}|request:{}\"",
            observed_basis_digest,
            operation_name.trim(),
            mutation_request_digest,
        );
        let request_identity_digest = format!(
            "compat-http-mutation-request-precondition-v1|basis:{}|if-match:{}",
            requested_basis_digest.as_deref().unwrap_or("none"),
            if_match.as_deref().unwrap_or("none"),
        );
        if let Some(expected_basis) = requested_basis_digest.as_deref() {
            if expected_basis != observed_basis_digest {
                return Err(WorthServerOperationReadinessDenial::new(
                    WorthServerOperationReadinessDenialCode::PreconditionFailed,
                    format!(
                        "compatibility mutation basis precondition `{expected_basis}` did not match the admitted mutation basis `{observed_basis_digest}`"
                    ),
                )
                .with_facts(
                    WorthServerOperationReadinessDenialFacts::default()
                        .with_basis_mismatch(expected_basis, observed_basis_digest),
                ));
            }
        }
        if let Some(expected_validator) = if_match.as_deref() {
            if expected_validator != validator {
                return Err(WorthServerOperationReadinessDenial::new(
                    WorthServerOperationReadinessDenialCode::PreconditionFailed,
                    format!(
                        "compatibility mutation validator precondition `{expected_validator}` did not match the canonical mutation validator `{validator}`"
                    ),
                )
                .with_facts(
                    WorthServerOperationReadinessDenialFacts::default()
                        .with_validator_mismatch(expected_validator, &validator),
                ));
            }
        }
        if let Some(expected_product_session) = expected_product_session.as_deref() {
            let Some(observed_product_session_identity) = observed_product_session_identity else {
                return Err(WorthServerOperationReadinessDenial::new(
                    WorthServerOperationReadinessDenialCode::PreconditionFailed,
                    format!(
                        "compatibility mutation product session precondition `{expected_product_session}` cannot be satisfied without an admitted product session identity"
                    ),
                ));
            };
            if expected_product_session != observed_product_session_identity {
                return Err(WorthServerOperationReadinessDenial::new(
                    WorthServerOperationReadinessDenialCode::PreconditionFailed,
                    format!(
                        "compatibility mutation product session precondition `{expected_product_session}` did not match the admitted session `{observed_product_session_identity}`"
                    ),
                ));
            }
        }
        if let Some(expected_binding) = idempotency_binding.as_deref() {
            let observed_binding =
                canonical_idempotency_binding(prepared_request, mutation_request_digest)?;
            if expected_binding != observed_binding {
                return Err(WorthServerOperationReadinessDenial::new(
                    WorthServerOperationReadinessDenialCode::PreconditionFailed,
                    format!(
                        "compatibility mutation idempotency binding `{expected_binding}` did not match the canonical binding `{observed_binding}`"
                    ),
                ));
            }
        }
        let canonical_digest = format!(
            "compat-http-mutation-precondition-v3|requested_basis:{}|if-match:{}|base_branch:{}|product_session:{}|idempotency_binding:{}|observed_basis:{}|validator:{}",
            requested_basis_digest.as_deref().unwrap_or("none"),
            if_match.as_deref().unwrap_or("none"),
            requested_base_branch.as_deref().unwrap_or("none"),
            expected_product_session.as_deref().unwrap_or("none"),
            idempotency_binding.as_deref().unwrap_or("none"),
            observed_basis_digest,
            validator,
        );
        Ok(Self {
            requested_basis_digest,
            if_match,
            observed_basis_digest: observed_basis_digest.to_string(),
            validator,
            request_identity_digest,
            canonical_digest,
        })
    }

    pub fn validator(&self) -> &str {
        &self.validator
    }

    pub fn requested_basis_digest(&self) -> Option<&str> {
        self.requested_basis_digest.as_deref()
    }

    pub fn observed_basis_digest(&self) -> &str {
        &self.observed_basis_digest
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }

    pub fn request_identity_digest(&self) -> &str {
        &self.request_identity_digest
    }
}

fn validate_requested_basis_digest(
    requested_basis_digest: Option<&str>,
) -> Result<(), WorthServerOperationReadinessDenial> {
    let Some(requested_basis_digest) = requested_basis_digest else {
        return Ok(());
    };
    let valid = requested_basis_digest.starts_with("basis:")
        && requested_basis_digest.len() > "basis:".len()
        && requested_basis_digest
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, ':' | '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(WorthServerOperationReadinessDenial::new(
            WorthServerOperationReadinessDenialCode::InvalidPreconditionInput,
            format!(
                "compatibility mutation basis precondition `{requested_basis_digest}` is not a canonical base digest"
            ),
        ))
    }
}

fn validate_requested_base_branch(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    requested_base_branch: Option<&str>,
) -> Result<(), WorthServerOperationReadinessDenial> {
    let Some(requested_base_branch) = requested_base_branch else {
        return Ok(());
    };
    let observed_branch = prepared_request
        .admission()
        .request_context()
        .branch_target()
        .canonical_label();
    if requested_base_branch == observed_branch {
        Ok(())
    } else {
        Err(WorthServerOperationReadinessDenial::new(
            WorthServerOperationReadinessDenialCode::PreconditionFailed,
            format!(
                "compatibility mutation base branch precondition `{requested_base_branch}` did not match the admitted branch `{observed_branch}`"
            ),
        ))
    }
}

fn validate_idempotency_binding_inputs(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    idempotency_binding: Option<&str>,
) -> Result<(), WorthServerOperationReadinessDenial> {
    let Some(_) = idempotency_binding else {
        return Ok(());
    };
    let Some(values) = prepared_request
        .request_contract()
        .canonical_headers()
        .values("idempotency-key")
    else {
        return Err(WorthServerOperationReadinessDenial::new(
            WorthServerOperationReadinessDenialCode::InvalidPreconditionInput,
            "compatibility mutation idempotency binding requires an `idempotency-key` header",
        ));
    };
    if values.len() == 1 && !values[0].trim().is_empty() {
        Ok(())
    } else {
        Err(WorthServerOperationReadinessDenial::new(
            WorthServerOperationReadinessDenialCode::InvalidPreconditionInput,
            "compatibility mutation idempotency binding requires exactly one canonical `idempotency-key` header value",
        ))
    }
}

fn canonical_idempotency_binding(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    mutation_request_digest: &str,
) -> Result<String, WorthServerOperationReadinessDenial> {
    let idempotency_key =
        read_single_header(prepared_request, "idempotency-key")?.ok_or_else(|| {
            WorthServerOperationReadinessDenial::new(
                WorthServerOperationReadinessDenialCode::InvalidPreconditionInput,
                "compatibility mutation idempotency binding requires an `idempotency-key` header",
            )
        })?;
    Ok(format!(
        "compat-http-idempotency-binding-v1|key:{}|request:{}",
        idempotency_key.trim(),
        mutation_request_digest,
    ))
}

fn read_single_query_pair(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    query_name: &str,
) -> Result<Option<String>, WorthServerOperationReadinessDenial> {
    let values = prepared_request
        .request_contract()
        .normalized_query_pairs()
        .iter()
        .filter_map(|(name, value)| (name == query_name).then_some(value.as_str()))
        .collect::<Vec<_>>();
    if values.len() > 1 {
        return Err(WorthServerOperationReadinessDenial::new(
            WorthServerOperationReadinessDenialCode::InvalidPreconditionInput,
            format!("compatibility mutation admits at most one `{query_name}` query value"),
        ));
    }
    Ok(values.first().map(|value| (*value).to_string()))
}

fn read_single_header(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    header_name: &str,
) -> Result<Option<String>, WorthServerOperationReadinessDenial> {
    let Some(values) = prepared_request
        .request_contract()
        .canonical_headers()
        .values(header_name)
    else {
        return Ok(None);
    };
    if values.len() != 1 {
        return Err(WorthServerOperationReadinessDenial::new(
            WorthServerOperationReadinessDenialCode::InvalidPreconditionInput,
            format!(
                "compatibility mutation requires a single canonical `{header_name}` header value"
            ),
        ));
    }
    Ok(Some(values[0].trim().to_string()))
}
