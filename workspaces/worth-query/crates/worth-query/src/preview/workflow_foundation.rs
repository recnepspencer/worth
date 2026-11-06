use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::execution::ExecutionCounters;
use crate::identity::{CanonicalQueryDigest, ValidatedQueryDigest};
use crate::preview::binding::PreviewSessionPlanBinding;
use crate::preview::comparison::PreviewComparisonShapeContract;
use crate::preview::evaluation::PreviewEvaluationClass;
use crate::preview::execution::PreviewExecutionCounters;
use crate::preview::workflow_context_identity;
use crate::WorthQueryEvidenceIdentity;
use crate::{WorthQueryEvidenceScope, WorthQueryEvidenceTag};
use worth_runtime_bridge::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewWorkflowFoundationArtifact {
    pub(in crate::preview) artifact_identity: WorthQueryEvidenceIdentity,
    pub(in crate::preview) binding_identity: WorthQueryEvidenceIdentity,
    pub(in crate::preview) canonical_query_digest: CanonicalQueryDigest,
    pub(in crate::preview) validated_query_digest: ValidatedQueryDigest,
    pub(in crate::preview) request_family: PreviewWorkflowFoundationRequest,
    pub(in crate::preview) preview_session_identity: BridgePreviewSessionIdentity,
    pub(in crate::preview) declaration_identity: BridgePreviewSessionDeclarationIdentity,
    pub(in crate::preview) declaration_digest_identity: WorthQueryEvidenceIdentity,
    pub(in crate::preview) lifecycle_state_kind: BridgePreviewLifecycleStateKind,
    pub(in crate::preview) evaluation_class: PreviewEvaluationClass,
    pub(in crate::preview) execution_record_identity: PreviewExecutionRecordIdentity,
    pub(in crate::preview) shape_check_width: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewWorkflowFoundationRequest {
    CompareBasisPair,
    DeferredMutationWriteback,
}

impl PreviewWorkflowFoundationRequest {
    pub fn compare_basis_pair() -> Self {
        Self::CompareBasisPair
    }

    pub fn deferred_mutation_writeback() -> Self {
        Self::DeferredMutationWriteback
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CompareBasisPair => "compare_basis_pair",
            Self::DeferredMutationWriteback => "deferred_mutation_writeback",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewWorkflowFoundationFailureClass {
    OutOfScopeWorkflowFoundationRequest,
    ReadOnlyPreviewWritebackFoundationForbidden,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewWorkflowFoundationError {
    failure_class: PreviewWorkflowFoundationFailureClass,
    message: &'static str,
    counters: PreviewExecutionCounters,
}

impl PreviewWorkflowFoundationError {
    pub fn failure_class(&self) -> &PreviewWorkflowFoundationFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn counters(&self) -> &PreviewExecutionCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedPreviewWorkflowFoundation {
    pub(in crate::preview) artifact: PreviewWorkflowFoundationArtifact,
    pub(in crate::preview) counters: PreviewExecutionCounters,
}

impl PreviewWorkflowFoundationArtifact {
    pub fn artifact_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.artifact_identity
    }

    pub fn artifact_for_reporting(&self) -> &str {
        self.artifact_identity.as_str()
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub fn binding_for_reporting(&self) -> &str {
        self.binding_identity.as_str()
    }

    pub fn request_family(&self) -> &PreviewWorkflowFoundationRequest {
        &self.request_family
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        &self.canonical_query_digest
    }

    pub fn validated_query_digest(&self) -> &ValidatedQueryDigest {
        &self.validated_query_digest
    }

    pub fn preview_session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.preview_session_identity
    }

    pub fn declaration_identity(&self) -> &BridgePreviewSessionDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn declaration_digest_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.declaration_digest_identity
    }

    pub fn declaration_digest_for_reporting(&self) -> &str {
        self.declaration_digest_identity.as_str()
    }

    pub fn lifecycle_state_kind(&self) -> BridgePreviewLifecycleStateKind {
        self.lifecycle_state_kind
    }

    pub fn evaluation_class(&self) -> &PreviewEvaluationClass {
        &self.evaluation_class
    }

    pub fn execution_record_identity(&self) -> &PreviewExecutionRecordIdentity {
        &self.execution_record_identity
    }

    pub fn shape_check_width(&self) -> usize {
        self.shape_check_width
    }
}

impl AdmittedPreviewWorkflowFoundation {
    pub fn artifact_identity(&self) -> &WorthQueryEvidenceIdentity {
        self.artifact.artifact_identity()
    }

    pub fn artifact_for_reporting(&self) -> &str {
        self.artifact.artifact_for_reporting()
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        self.artifact.binding_identity()
    }

    pub fn binding_for_reporting(&self) -> &str {
        self.artifact.binding_for_reporting()
    }

    pub fn request_family(&self) -> &PreviewWorkflowFoundationRequest {
        self.artifact.request_family()
    }

    pub fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        self.artifact.canonical_query_digest()
    }

    pub fn validated_query_digest(&self) -> &ValidatedQueryDigest {
        self.artifact.validated_query_digest()
    }

    pub fn preview_session_identity(&self) -> &BridgePreviewSessionIdentity {
        self.artifact.preview_session_identity()
    }

    pub fn declaration_identity(&self) -> &BridgePreviewSessionDeclarationIdentity {
        self.artifact.declaration_identity()
    }

    pub fn declaration_digest_identity(&self) -> &WorthQueryEvidenceIdentity {
        self.artifact.declaration_digest_identity()
    }

    pub fn declaration_digest_for_reporting(&self) -> &str {
        self.artifact.declaration_digest_for_reporting()
    }

    pub fn lifecycle_state_kind(&self) -> BridgePreviewLifecycleStateKind {
        self.artifact.lifecycle_state_kind()
    }

    pub fn evaluation_class(&self) -> &PreviewEvaluationClass {
        self.artifact.evaluation_class()
    }

    pub fn execution_record_identity(&self) -> &PreviewExecutionRecordIdentity {
        self.artifact.execution_record_identity()
    }

    pub fn shape_check_width(&self) -> usize {
        self.artifact.shape_check_width()
    }

    pub fn counters(&self) -> &PreviewExecutionCounters {
        &self.counters
    }

    pub(crate) fn artifact(&self) -> &PreviewWorkflowFoundationArtifact {
        &self.artifact
    }
}

fn derive_preview_workflow_foundation(
    binding: &PreviewSessionPlanBinding,
    request: PreviewWorkflowFoundationRequest,
) -> Result<PreviewWorkflowFoundationArtifact, IdentityDerivationDenial> {
    let binding_tuple = binding.basis().binding_tuple();
    let execution_record_identity = binding_tuple
        .execution_record_identity()
        .cloned()
        .expect("active preview bindings must carry an execution record identity");
    let binding_identity =
        workflow_context_identity::compose_preview_binding_tuple_workflow_identity(binding_tuple)?;
    let declaration_digest_identity =
        workflow_context_identity::compose_preview_declaration_digest_workflow_identity(
            binding_tuple,
        )?;
    let artifact_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_workflow_foundation_artifact_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_identity)
            .field_shape(WorthQueryEvidenceTag::new("request"), request.as_str())
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("preview_session"),
                &binding_tuple
                    .preview_session_identity()
                    .bridge_admission_evidence(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_identity"),
                &binding_tuple
                    .declaration_identity()
                    .bridge_admission_evidence(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_digest"),
                &declaration_digest_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                workflow_context_identity::preview_lifecycle_state_label(
                    binding_tuple.lifecycle_state_kind(),
                ),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("execution_record"),
                &execution_record_identity.bridge_admission_evidence(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("evaluation_class"),
                binding_tuple.evaluation_class().as_str(),
            )
            .seal()?;

    Ok(PreviewWorkflowFoundationArtifact {
        artifact_identity,
        binding_identity,
        canonical_query_digest: binding_tuple.canonical_query_digest().clone(),
        validated_query_digest: binding_tuple.validated_query_digest().clone(),
        request_family: request,
        preview_session_identity: binding_tuple.preview_session_identity().clone(),
        declaration_identity: binding_tuple.declaration_identity().clone(),
        declaration_digest_identity,
        lifecycle_state_kind: binding_tuple.lifecycle_state_kind(),
        evaluation_class: binding_tuple.evaluation_class().clone(),
        execution_record_identity,
        shape_check_width: PreviewComparisonShapeContract::from_preflight(binding.preflight())?
            .shape_check_width,
    })
}

pub fn admit_preview_workflow_foundation(
    binding: &PreviewSessionPlanBinding,
) -> Result<AdmittedPreviewWorkflowFoundation, PreviewWorkflowFoundationError> {
    admit_preview_workflow_foundation_request(
        binding,
        PreviewWorkflowFoundationRequest::compare_basis_pair(),
    )
}

pub fn admit_preview_workflow_foundation_request(
    binding: &PreviewSessionPlanBinding,
    request: PreviewWorkflowFoundationRequest,
) -> Result<AdmittedPreviewWorkflowFoundation, PreviewWorkflowFoundationError> {
    if request == PreviewWorkflowFoundationRequest::DeferredMutationWriteback
        && binding.basis().binding_tuple().evaluation_class()
            == &PreviewEvaluationClass::read_only()
    {
        return Err(PreviewWorkflowFoundationError {
            failure_class:
                PreviewWorkflowFoundationFailureClass::ReadOnlyPreviewWritebackFoundationForbidden,
            message:
                "read-only preview workflow foundations cannot request deferred mutation writeback authority",
            counters: PreviewExecutionCounters {
                binding_counters: binding.report().counters().clone(),
                execution_counters: ExecutionCounters::default(),
                preview_execution_envelope_count: 0,
                preview_execution_count: 0,
                preview_promotable_execution_count: 0,
                preview_read_only_execution_count: 0,
                preview_comparison_eligibility_proof_count: 0,
                preview_comparison_shape_check_width: 0,
                preview_workflow_foundation_admission_count: 0,
                preview_workflow_foundation_denial_count: 1,
                preview_workflow_foundation_artifact_lookup_count: 0,
                preview_work_avoided_by_explicit_basis_count: 0,
            },
        });
    }

    Ok(AdmittedPreviewWorkflowFoundation {
        artifact: derive_preview_workflow_foundation(binding, request),
        counters: PreviewExecutionCounters {
            binding_counters: binding.report().counters().clone(),
            execution_counters: ExecutionCounters::default(),
            preview_execution_envelope_count: 0,
            preview_execution_count: 0,
            preview_promotable_execution_count: 0,
            preview_read_only_execution_count: 0,
            preview_comparison_eligibility_proof_count: 0,
            preview_comparison_shape_check_width: 0,
            preview_workflow_foundation_admission_count: 1,
            preview_workflow_foundation_denial_count: 0,
            preview_workflow_foundation_artifact_lookup_count: 1,
            preview_work_avoided_by_explicit_basis_count: 1,
        },
    })
}
