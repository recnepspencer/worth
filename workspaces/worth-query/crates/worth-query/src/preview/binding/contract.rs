use crate::basis::ExecutionPreflightBundle;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::{
    CanonicalQueryDigest, CanonicalResultShapeDigest, ValidatedQueryDigest,
    ValidatedResultShapeDigest,
};
use crate::preview::binding::{PreviewBindingCounters, PreviewComplexityContract};
use crate::preview::evaluation::PreviewEvaluationClass;
use crate::preview::session_context::PreviewSessionQueryContext;
use crate::preview::workflow_context_identity;
use worth_runtime_bridge::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewLifecycleMetadata {
    lifecycle_state_kind: BridgePreviewLifecycleStateKind,
    execution_record_identity: Option<PreviewExecutionRecordIdentity>,
    replay_bundle_digest: Option<String>,
    promotion_record_identity: Option<String>,
    promotion_proof_digest: Option<String>,
}

impl PreviewLifecycleMetadata {
    pub(super) fn from_source(
        lifecycle_state_kind: BridgePreviewLifecycleStateKind,
        execution_record_identity: Option<PreviewExecutionRecordIdentity>,
    ) -> Self {
        Self {
            lifecycle_state_kind,
            execution_record_identity,
            replay_bundle_digest: None,
            promotion_record_identity: None,
            promotion_proof_digest: None,
        }
    }
    pub fn lifecycle_state_kind(&self) -> BridgePreviewLifecycleStateKind {
        self.lifecycle_state_kind
    }

    pub fn execution_record_identity(&self) -> Option<&PreviewExecutionRecordIdentity> {
        self.execution_record_identity.as_ref()
    }

    pub fn replay_bundle_digest(&self) -> Option<&str> {
        self.replay_bundle_digest.as_deref()
    }

    pub fn promotion_record_identity(&self) -> Option<&str> {
        self.promotion_record_identity.as_deref()
    }

    pub fn promotion_proof_digest(&self) -> Option<&str> {
        self.promotion_proof_digest.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewSessionBindingTuple {
    digest: String,
    canonical_query_digest: CanonicalQueryDigest,
    canonical_result_shape_digest: CanonicalResultShapeDigest,
    validated_query_digest: ValidatedQueryDigest,
    validated_result_shape_digest: ValidatedResultShapeDigest,
    evaluation_class: PreviewEvaluationClass,
    preview_session_identity: BridgePreviewSessionIdentity,
    declaration_identity: BridgePreviewSessionDeclarationIdentity,
    declaration_digest: String,
    lifecycle_state_kind: BridgePreviewLifecycleStateKind,
    execution_record_identity: Option<PreviewExecutionRecordIdentity>,
    replay_bundle_digest: Option<String>,
    promotion_record_identity: Option<String>,
    promotion_proof_digest: Option<String>,
}

impl PreviewSessionBindingTuple {
    pub(super) fn from_admitted(
        preflight: &ExecutionPreflightBundle,
        query_context: &PreviewSessionQueryContext,
    ) -> Result<Self, IdentityDerivationDenial> {
        let source = query_context.source.snapshot();
        let mut binding = Self {
            digest: String::new(),
            canonical_query_digest: preflight.plan().query().canonical_query_digest().clone(),
            canonical_result_shape_digest: preflight
                .plan()
                .result_shape()
                .canonical_result_shape_digest()
                .clone(),
            validated_query_digest: preflight.plan().query().validated_query_digest().clone(),
            validated_result_shape_digest: preflight
                .plan()
                .result_shape()
                .validated_result_shape_digest()
                .clone(),
            evaluation_class: query_context.evaluation_class.clone(),
            preview_session_identity: source.preview_session_identity.clone(),
            declaration_identity: source.declaration_identity.clone(),
            declaration_digest: source.declaration_digest.clone(),
            lifecycle_state_kind: source.lifecycle_state_kind,
            execution_record_identity: source.execution_record_identity.clone(),
            replay_bundle_digest: None,
            promotion_record_identity: None,
            promotion_proof_digest: None,
        };
        binding.digest =
            workflow_context_identity::compose_preview_session_binding_tuple_digest(&binding)?;
        Ok(binding)
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        &self.canonical_query_digest
    }

    pub fn canonical_result_shape_digest(&self) -> &CanonicalResultShapeDigest {
        &self.canonical_result_shape_digest
    }

    pub fn validated_query_digest(&self) -> &ValidatedQueryDigest {
        &self.validated_query_digest
    }

    pub fn validated_result_shape_digest(&self) -> &ValidatedResultShapeDigest {
        &self.validated_result_shape_digest
    }

    pub fn evaluation_class(&self) -> &PreviewEvaluationClass {
        &self.evaluation_class
    }

    pub fn preview_session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.preview_session_identity
    }

    pub fn declaration_identity(&self) -> &BridgePreviewSessionDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn lifecycle_state_kind(&self) -> BridgePreviewLifecycleStateKind {
        self.lifecycle_state_kind
    }

    pub fn execution_record_identity(&self) -> Option<&PreviewExecutionRecordIdentity> {
        self.execution_record_identity.as_ref()
    }

    pub fn replay_bundle_digest(&self) -> Option<&str> {
        self.replay_bundle_digest.as_deref()
    }

    pub fn promotion_record_identity(&self) -> Option<&str> {
        self.promotion_record_identity.as_deref()
    }

    pub fn promotion_proof_digest(&self) -> Option<&str> {
        self.promotion_proof_digest.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewSessionBasis {
    binding_tuple: PreviewSessionBindingTuple,
}

impl PreviewSessionBasis {
    pub(super) fn new(binding_tuple: PreviewSessionBindingTuple) -> Self {
        Self { binding_tuple }
    }

    pub fn binding_tuple(&self) -> &PreviewSessionBindingTuple {
        &self.binding_tuple
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewBindingReport {
    binding_digest: String,
    evaluation_class: PreviewEvaluationClass,
    basis_binding_contract: PreviewComplexityContract,
    execution_metadata_contract: PreviewComplexityContract,
    counters: PreviewBindingCounters,
}

impl PreviewBindingReport {
    pub(super) fn new(
        binding_digest: String,
        evaluation_class: PreviewEvaluationClass,
        counters: PreviewBindingCounters,
    ) -> Self {
        Self {
            binding_digest,
            evaluation_class,
            basis_binding_contract: PreviewComplexityContract::preview_basis_binding_contract(),
            execution_metadata_contract:
                PreviewComplexityContract::preview_execution_metadata_contract(),
            counters,
        }
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    pub fn evaluation_class(&self) -> &PreviewEvaluationClass {
        &self.evaluation_class
    }

    pub fn basis_binding_contract(&self) -> &PreviewComplexityContract {
        &self.basis_binding_contract
    }

    pub fn execution_metadata_contract(&self) -> &PreviewComplexityContract {
        &self.execution_metadata_contract
    }

    pub fn counters(&self) -> &PreviewBindingCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewSessionPlanBinding {
    preflight: ExecutionPreflightBundle,
    query_context: PreviewSessionQueryContext,
    basis: PreviewSessionBasis,
    lifecycle_metadata: PreviewLifecycleMetadata,
    report: PreviewBindingReport,
}

impl PreviewSessionPlanBinding {
    pub(super) fn from_admitted(
        preflight: ExecutionPreflightBundle,
        query_context: PreviewSessionQueryContext,
        basis: PreviewSessionBasis,
        lifecycle_metadata: PreviewLifecycleMetadata,
        report: PreviewBindingReport,
    ) -> Self {
        Self {
            preflight,
            query_context,
            basis,
            lifecycle_metadata,
            report,
        }
    }
    pub fn preflight(&self) -> &ExecutionPreflightBundle {
        &self.preflight
    }

    pub fn query_context(&self) -> &PreviewSessionQueryContext {
        &self.query_context
    }

    pub fn basis(&self) -> &PreviewSessionBasis {
        &self.basis
    }

    pub fn lifecycle_metadata(&self) -> &PreviewLifecycleMetadata {
        &self.lifecycle_metadata
    }

    pub fn report(&self) -> &PreviewBindingReport {
        &self.report
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadOnlyPreviewSessionPlanBinding {
    inner: PreviewSessionPlanBinding,
}

impl ReadOnlyPreviewSessionPlanBinding {
    pub(in crate::preview) fn from_admitted(inner: PreviewSessionPlanBinding) -> Self {
        Self { inner }
    }
    pub fn preflight(&self) -> &ExecutionPreflightBundle {
        self.inner.preflight()
    }

    pub fn query_context(&self) -> &PreviewSessionQueryContext {
        self.inner.query_context()
    }

    pub fn basis(&self) -> &PreviewSessionBasis {
        self.inner.basis()
    }

    pub fn lifecycle_metadata(&self) -> &PreviewLifecycleMetadata {
        self.inner.lifecycle_metadata()
    }

    pub fn report(&self) -> &PreviewBindingReport {
        self.inner.report()
    }

    pub(crate) fn as_preview_binding(&self) -> &PreviewSessionPlanBinding {
        &self.inner
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionEligiblePreviewSessionPlanBinding {
    inner: PreviewSessionPlanBinding,
}

impl PromotionEligiblePreviewSessionPlanBinding {
    pub(in crate::preview) fn from_admitted(inner: PreviewSessionPlanBinding) -> Self {
        Self { inner }
    }
    pub fn preflight(&self) -> &ExecutionPreflightBundle {
        self.inner.preflight()
    }

    pub fn query_context(&self) -> &PreviewSessionQueryContext {
        self.inner.query_context()
    }

    pub fn basis(&self) -> &PreviewSessionBasis {
        self.inner.basis()
    }

    pub fn lifecycle_metadata(&self) -> &PreviewLifecycleMetadata {
        self.inner.lifecycle_metadata()
    }

    pub fn report(&self) -> &PreviewBindingReport {
        self.inner.report()
    }

    pub(crate) fn as_preview_binding(&self) -> &PreviewSessionPlanBinding {
        &self.inner
    }
}
