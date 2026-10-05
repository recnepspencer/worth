#[derive(Clone, Debug)]
pub struct WorthServerProductOperationInput {
    operation_name: String,
    payload: super::WorthServerProductOperationPayload,
    snapshot_precondition: Option<crate::WorthServerProductSnapshotPrecondition>,
    idempotency_key: Option<crate::WorthServerProductIdempotencyKey>,
    product_session_identity: Option<String>,
}

impl WorthServerProductOperationInput {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.operation_name.capacity() as u64)
            .saturating_add(self.payload.owned_allocation_capacity_bytes())
            .saturating_add(
                self.snapshot_precondition
                    .as_ref()
                    .map_or(0, |precondition| {
                        (precondition.base_digest().value().len()
                            + precondition.canonical_digest().len()) as u64
                    }),
            )
            .saturating_add(self.idempotency_key.as_ref().map_or(0, |key| {
                (key.value().len() + key.canonical_digest().len()) as u64
            }))
            .saturating_add(
                self.product_session_identity
                    .as_ref()
                    .map_or(0, |id| id.capacity() as u64),
            )
    }

    pub fn new(
        operation_name: impl Into<String>,
        payload: super::WorthServerProductOperationPayload,
    ) -> Self {
        Self {
            operation_name: operation_name.into(),
            payload,
            snapshot_precondition: None,
            idempotency_key: None,
            product_session_identity: None,
        }
    }

    pub fn with_basis_digest(mut self, basis_digest: impl Into<String>) -> Self {
        let canonical_basis_digest =
            crate::WorthServerProductOperationBaseDigest::canonicalize_text(basis_digest.into())
                .expect("basis digest shim should remain canonical");
        self.snapshot_precondition = Some(
            crate::WorthServerProductSnapshotPrecondition::at_base_digest(
                crate::WorthServerProductOperationBaseDigest::new(canonical_basis_digest)
                    .expect("basis digest shim should remain canonical"),
            ),
        );
        self
    }

    pub fn with_snapshot_precondition(
        mut self,
        snapshot_precondition: crate::WorthServerProductSnapshotPrecondition,
    ) -> Self {
        self.snapshot_precondition = Some(snapshot_precondition);
        self
    }

    pub fn with_idempotency_key(
        mut self,
        idempotency_key: crate::WorthServerProductIdempotencyKey,
    ) -> Self {
        self.idempotency_key = Some(idempotency_key);
        self
    }

    pub fn with_product_session_identity(
        mut self,
        product_session_identity: impl Into<String>,
    ) -> Self {
        self.product_session_identity = Some(product_session_identity.into());
        self
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub(crate) fn payload(&self) -> &super::WorthServerProductOperationPayload {
        &self.payload
    }

    pub fn snapshot_precondition(&self) -> Option<&crate::WorthServerProductSnapshotPrecondition> {
        self.snapshot_precondition.as_ref()
    }

    pub(crate) fn basis_digest(&self) -> Option<&str> {
        self.snapshot_precondition
            .as_ref()
            .map(|precondition| precondition.base_digest().value())
    }

    pub(crate) fn idempotency_key(&self) -> Option<&crate::WorthServerProductIdempotencyKey> {
        self.idempotency_key.as_ref()
    }

    pub(crate) fn product_session_identity(&self) -> Option<&str> {
        self.product_session_identity.as_deref()
    }

    pub(crate) fn into_payload(self) -> super::WorthServerProductOperationPayload {
        self.payload
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerProductOperationExecutionBoundary {
    RejectedBeforeAdapterExecution,
    AdapterExecutionAttempted,
    DurableExecutorAttempted,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthServerProductOperationSurfaceDenialFacts {
    readiness_code: Option<crate::WorthServerOperationReadinessDenialCode>,
    stale_basis_denial: Option<crate::WorthServerProductStaleBasisDenial>,
    rebase_required: Option<crate::WorthServerProductRebaseRequired>,
    session_denial_code: Option<crate::WorthServerProductSessionDenialCode>,
    idempotency_conflict: Option<crate::WorthServerProductIdempotencyConflict>,
    execution_boundary: Option<WorthServerProductOperationExecutionBoundary>,
    recovery_handle: Option<crate::WorthServerDurableProductMutationRecoveryHandle>,
}

impl WorthServerProductOperationSurfaceDenialFacts {
    pub fn readiness_code(&self) -> Option<crate::WorthServerOperationReadinessDenialCode> {
        self.readiness_code.clone()
    }

    pub fn expected_basis_digest(&self) -> Option<&str> {
        self.stale_basis_denial
            .as_ref()
            .map(crate::WorthServerProductStaleBasisDenial::expected_base_digest)
    }

    pub fn observed_basis_digest(&self) -> Option<&str> {
        self.stale_basis_denial
            .as_ref()
            .map(crate::WorthServerProductStaleBasisDenial::observed_base_digest)
    }

    pub fn session_denial_code(&self) -> Option<crate::WorthServerProductSessionDenialCode> {
        self.session_denial_code
    }

    pub fn stale_basis_denial(&self) -> Option<&crate::WorthServerProductStaleBasisDenial> {
        self.stale_basis_denial.as_ref()
    }

    pub fn rebase_required(&self) -> Option<&crate::WorthServerProductRebaseRequired> {
        self.rebase_required.as_ref()
    }

    pub fn idempotency_conflict(&self) -> Option<&crate::WorthServerProductIdempotencyConflict> {
        self.idempotency_conflict.as_ref()
    }

    pub fn execution_boundary(&self) -> Option<&WorthServerProductOperationExecutionBoundary> {
        self.execution_boundary.as_ref()
    }

    pub fn recovery_handle(
        &self,
    ) -> Option<&crate::WorthServerDurableProductMutationRecoveryHandle> {
        self.recovery_handle.as_ref()
    }

    pub(crate) fn with_readiness_code(
        mut self,
        readiness_code: crate::WorthServerOperationReadinessDenialCode,
    ) -> Self {
        self.readiness_code = Some(readiness_code);
        self
    }

    pub(crate) fn with_basis_mismatch(
        mut self,
        stale_basis_denial: crate::WorthServerProductStaleBasisDenial,
    ) -> Self {
        self.rebase_required = Some(crate::WorthServerProductRebaseRequired::new(
            stale_basis_denial.clone(),
        ));
        self.stale_basis_denial = Some(stale_basis_denial);
        self
    }

    pub(crate) fn with_session_denial_code(
        mut self,
        session_denial_code: crate::WorthServerProductSessionDenialCode,
    ) -> Self {
        self.session_denial_code = Some(session_denial_code);
        self
    }

    pub(crate) fn with_execution_boundary(
        mut self,
        execution_boundary: WorthServerProductOperationExecutionBoundary,
    ) -> Self {
        self.execution_boundary = Some(execution_boundary);
        self
    }

    pub(crate) fn with_idempotency_conflict(
        mut self,
        idempotency_conflict: crate::WorthServerProductIdempotencyConflict,
    ) -> Self {
        self.idempotency_conflict = Some(idempotency_conflict);
        self
    }

    pub(crate) fn with_recovery_handle(
        mut self,
        recovery_handle: crate::WorthServerDurableProductMutationRecoveryHandle,
    ) -> Self {
        self.recovery_handle = Some(recovery_handle);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductOperationSurfaceDenial {
    code: WorthServerProductOperationSurfaceDenialCode,
    detail: String,
    facts: Option<WorthServerProductOperationSurfaceDenialFacts>,
}

impl WorthServerProductOperationSurfaceDenial {
    pub(crate) fn new(code: WorthServerProductOperationSurfaceDenialCode, detail: String) -> Self {
        Self {
            code,
            detail,
            facts: None,
        }
    }

    pub(crate) fn from_request_denial(denial: crate::WorthServerOperationRequestDenial) -> Self {
        Self::new(
            WorthServerProductOperationSurfaceDenialCode::RequestDenied,
            denial.detail().to_string(),
        )
        .with_facts(
            WorthServerProductOperationSurfaceDenialFacts::default().with_execution_boundary(
                WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
            ),
        )
    }

    pub(crate) fn from_admission_denial(
        denial: crate::WorthServerOperationAdmissionDenial,
    ) -> Self {
        Self::new(
            WorthServerProductOperationSurfaceDenialCode::AdmissionDenied,
            denial.detail().to_string(),
        )
        .with_facts(
            WorthServerProductOperationSurfaceDenialFacts::default().with_execution_boundary(
                WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
            ),
        )
    }

    pub(crate) fn from_readiness_denial(
        denial: crate::WorthServerOperationReadinessDenial,
    ) -> Self {
        let facts = denial.facts();
        let mut surface_facts = WorthServerProductOperationSurfaceDenialFacts::default()
            .with_readiness_code(denial.code())
            .with_execution_boundary(
                WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
            );
        if let Some(facts) = facts {
            if let (Some(expected_basis_digest), Some(observed_basis_digest)) =
                (facts.expected_basis_digest(), facts.observed_basis_digest())
            {
                surface_facts = surface_facts.with_basis_mismatch(
                    crate::WorthServerProductStaleBasisDenial::new(
                        expected_basis_digest,
                        observed_basis_digest,
                    ),
                );
            }
        }
        Self::new(
            WorthServerProductOperationSurfaceDenialCode::ReadinessDenied,
            denial.detail().to_string(),
        )
        .with_facts(surface_facts)
    }

    pub(crate) fn with_facts(
        mut self,
        facts: WorthServerProductOperationSurfaceDenialFacts,
    ) -> Self {
        self.facts = Some(facts);
        self
    }

    pub fn code(&self) -> WorthServerProductOperationSurfaceDenialCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn facts(&self) -> Option<&WorthServerProductOperationSurfaceDenialFacts> {
        self.facts.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerProductOperationSurfaceDenialCode {
    UnknownOperationName,
    RequestDenied,
    AdmissionDenied,
    ReadinessDenied,
    PreconditionDenied,
    IdempotencyConflict,
    InvalidDeclaration,
    InvalidDurableCompletion,
    InvalidResultArtifact,
    Indeterminate,
}
