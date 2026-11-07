use worth_foundational::DiagnosticRichnessProfile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerQueryHandoffDenial {
    code: WorthServerQueryHandoffDenialCode,
    diagnostics_profile: DiagnosticRichnessProfile,
    detail: String,
    facts: Option<WorthServerQueryHandoffDenialFacts>,
    pub(crate) abuse_budget_receipt: Option<crate::WorthServerAbuseBudgetReceipt>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerQueryHandoffDenialFamily {
    Authority,
    Request,
    Support,
    Precondition,
    Runtime,
}

impl WorthServerQueryHandoffDenial {
    pub(crate) fn new(
        code: WorthServerQueryHandoffDenialCode,
        diagnostics_profile: DiagnosticRichnessProfile,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            diagnostics_profile,
            detail: detail.into(),
            facts: None,
            abuse_budget_receipt: None,
        }
    }

    pub(crate) fn with_facts(mut self, facts: WorthServerQueryHandoffDenialFacts) -> Self {
        self.facts = Some(facts);
        self
    }

    pub(crate) fn with_abuse_budget_receipt(
        mut self,
        abuse_budget_receipt: crate::WorthServerAbuseBudgetReceipt,
    ) -> Self {
        self.abuse_budget_receipt = Some(abuse_budget_receipt);
        self
    }

    pub fn code(&self) -> WorthServerQueryHandoffDenialCode {
        self.code
    }

    pub fn diagnostics_profile(&self) -> DiagnosticRichnessProfile {
        self.diagnostics_profile
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn abuse_budget_receipt(&self) -> Option<&crate::WorthServerAbuseBudgetReceipt> {
        self.abuse_budget_receipt.as_ref()
    }

    pub fn facts(&self) -> Option<&WorthServerQueryHandoffDenialFacts> {
        self.facts.as_ref()
    }

    pub fn family(&self) -> WorthServerQueryHandoffDenialFamily {
        match self.code {
            WorthServerQueryHandoffDenialCode::AuthorityDenied
            | WorthServerQueryHandoffDenialCode::AuthorizationDenied
            | WorthServerQueryHandoffDenialCode::OperationFamilyNotRegistered
            | WorthServerQueryHandoffDenialCode::OperationFamilyDisabled
            | WorthServerQueryHandoffDenialCode::OperationFamilyNotExposedOnSurface => {
                WorthServerQueryHandoffDenialFamily::Authority
            }
            WorthServerQueryHandoffDenialCode::CompatibilityBasisRequestInvalid
            | WorthServerQueryHandoffDenialCode::CompatibilityBasisRequestUnsupported
            | WorthServerQueryHandoffDenialCode::CompatibilityConditionalRequestInvalid
            | WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid
            | WorthServerQueryHandoffDenialCode::CompatibilityStreamingRequestInvalid
            | WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
            | WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid
            | WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyUnsupported
            | WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyForbidden
            | WorthServerQueryHandoffDenialCode::UnknownOperationName
            | WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid
            | WorthServerQueryHandoffDenialCode::DirectDeclarationSourceNotAdmitted
            | WorthServerQueryHandoffDenialCode::DirectProjectionBindingInvalid
            | WorthServerQueryHandoffDenialCode::DirectMutationBindingDenied
            | WorthServerQueryHandoffDenialCode::DirectMutationContractDenied
            | WorthServerQueryHandoffDenialCode::DirectMutationAssertionDenied
            | WorthServerQueryHandoffDenialCode::DirectMutationContinuityDenied
            | WorthServerQueryHandoffDenialCode::DirectMutationNamingDenied
            | WorthServerQueryHandoffDenialCode::DirectMutationTargetReferenceDenied => {
                WorthServerQueryHandoffDenialFamily::Request
            }
            WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
            | WorthServerQueryHandoffDenialCode::DownstreamDeliveryRequiresReadIntent
            | WorthServerQueryHandoffDenialCode::RuntimeBackedResumeUnsupported
            | WorthServerQueryHandoffDenialCode::DurableResumeDeferred
            | WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionDenied
            | WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionDeferred
            | WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionSourceMismatch => {
                WorthServerQueryHandoffDenialFamily::Support
            }
            WorthServerQueryHandoffDenialCode::CompatibilityConditionalReadPreconditionFailed
            | WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
            | WorthServerQueryHandoffDenialCode::CompatibilityIdempotencyConflict
            | WorthServerQueryHandoffDenialCode::LeaseDeclarationContextMismatch
            | WorthServerQueryHandoffDenialCode::RuntimeBackedResumeMissingBasis
            | WorthServerQueryHandoffDenialCode::RuntimeBackedResumeStaleBasis => {
                WorthServerQueryHandoffDenialFamily::Precondition
            }
            WorthServerQueryHandoffDenialCode::PreparedIntentMismatch
            | WorthServerQueryHandoffDenialCode::CompatibilityConditionalReadNotModified
            | WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed
            | WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable => {
                WorthServerQueryHandoffDenialFamily::Runtime
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthServerQueryHandoffDenialFacts {
    rejected_operation_name: Option<String>,
    expected_basis_digest: Option<String>,
    observed_basis_digest: Option<String>,
    expected_validator: Option<String>,
    observed_validator: Option<String>,
    idempotency_key: Option<String>,
    conflicting_request_digest: Option<String>,
    bound_request_digest: Option<String>,
}

impl WorthServerQueryHandoffDenialFacts {
    pub fn rejected_operation_name(&self) -> Option<&str> {
        self.rejected_operation_name.as_deref()
    }

    pub fn expected_basis_digest(&self) -> Option<&str> {
        self.expected_basis_digest.as_deref()
    }

    pub fn observed_basis_digest(&self) -> Option<&str> {
        self.observed_basis_digest.as_deref()
    }

    pub fn expected_validator(&self) -> Option<&str> {
        self.expected_validator.as_deref()
    }

    pub fn observed_validator(&self) -> Option<&str> {
        self.observed_validator.as_deref()
    }

    pub fn idempotency_key(&self) -> Option<&str> {
        self.idempotency_key.as_deref()
    }

    pub fn conflicting_request_digest(&self) -> Option<&str> {
        self.conflicting_request_digest.as_deref()
    }

    pub fn bound_request_digest(&self) -> Option<&str> {
        self.bound_request_digest.as_deref()
    }

    pub(crate) fn with_rejected_operation_name(
        mut self,
        operation_name: impl Into<String>,
    ) -> Self {
        self.rejected_operation_name = Some(operation_name.into());
        self
    }

    pub(crate) fn with_basis_mismatch(
        mut self,
        expected_basis_digest: impl Into<String>,
        observed_basis_digest: impl Into<String>,
    ) -> Self {
        self.expected_basis_digest = Some(expected_basis_digest.into());
        self.observed_basis_digest = Some(observed_basis_digest.into());
        self
    }

    pub(crate) fn with_validator_mismatch(
        mut self,
        expected_validator: impl Into<String>,
        observed_validator: impl Into<String>,
    ) -> Self {
        self.expected_validator = Some(expected_validator.into());
        self.observed_validator = Some(observed_validator.into());
        self
    }

    pub(crate) fn with_idempotency_conflict(
        mut self,
        idempotency_key: impl Into<String>,
        conflicting_request_digest: impl Into<String>,
        bound_request_digest: impl Into<String>,
    ) -> Self {
        self.idempotency_key = Some(idempotency_key.into());
        self.conflicting_request_digest = Some(conflicting_request_digest.into());
        self.bound_request_digest = Some(bound_request_digest.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerQueryHandoffDenialCode {
    EvidenceIdentityDerivationDenied(
        worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    AuthorityDenied,
    AuthorizationDenied,
    OperationFamilyNotRegistered,
    OperationFamilyDisabled,
    OperationFamilyNotExposedOnSurface,
    PreparedIntentMismatch,
    UnsupportedQueryFacadeFamily,
    DownstreamDeliveryRequiresReadIntent,
    DirectDeclarationBindingInvalid,
    DirectDeclarationSourceNotAdmitted,
    CompatibilityBasisRequestInvalid,
    CompatibilityBasisRequestUnsupported,
    CompatibilityConditionalRequestInvalid,
    CompatibilityConditionalReadNotModified,
    CompatibilityConditionalReadPreconditionFailed,
    CompatibilityDownloadRequestInvalid,
    CompatibilityStreamingRequestInvalid,
    CompatibilityUploadRequestInvalid,
    CompatibilityMutationRequestInvalid,
    CompatibilityMutationFamilyUnsupported,
    CompatibilityMutationFamilyForbidden,
    CompatibilityMutationPreconditionFailed,
    CompatibilityIdempotencyConflict,
    UnknownOperationName,
    LeaseDeclarationContextMismatch,
    RuntimeBackedResumeMissingBasis,
    RuntimeBackedResumeStaleBasis,
    RuntimeBackedResumeUnsupported,
    DurableResumeDeferred,
    WorkspaceBindingFailed,
    RetainedQueryArtifactUnavailable,
    DirectProjectionBindingInvalid,
    DirectMutationBindingDenied,
    DirectMutationContractDenied,
    DirectMutationAssertionDenied,
    DirectMutationContinuityDenied,
    DirectMutationNamingDenied,
    DirectMutationTargetReferenceDenied,
    ProjectionFactConsumptionDenied,
    ProjectionFactConsumptionDeferred,
    ProjectionFactConsumptionSourceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerQueryHandoffDeferred {
    reason: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerQueryHandoffStale {
    reason: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerQueryHandoffRebindRequired {
    reason: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerQueryHandoffFailure {
    reason: &'static str,
}

impl WorthServerQueryHandoffFailure {
    pub(crate) fn new(reason: &'static str) -> Self {
        Self { reason }
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}
