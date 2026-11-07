use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::execution::{ExecutionError, ExecutionResultEnvelope};
use crate::identity::{ResultDigest, ValidatedQueryDigest};
use crate::preview::binding::{
    PreviewLifecycleMetadata, PreviewSessionBasis, PreviewSessionPlanBinding,
};
use crate::preview::comparison::PreviewComparisonEligibilityArtifact;
use crate::preview::evaluation::PreviewEvaluationClass;
use crate::preview::execution::PreviewExecutionCounters;
use crate::preview::workflow_foundation::{
    AdmittedPreviewWorkflowFoundation, PreviewWorkflowFoundationArtifact,
};
use worth_runtime_bridge::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewExecutionReport {
    pub(in crate::preview) preview_execution_digest: String,
    pub(in crate::preview) binding_digest: String,
    pub(super) basis_digest: String,
    pub(super) preview_session_identity: BridgePreviewSessionIdentity,
    pub(super) lifecycle_state_kind: BridgePreviewLifecycleStateKind,
    pub(super) execution_record_identity: PreviewExecutionRecordIdentity,
    pub(in crate::preview) query_digest: ValidatedQueryDigest,
    pub(in crate::preview) result_digest: ResultDigest,
    pub(super) comparison_eligibility_digest: String,
    pub(super) workflow_foundation_digest: String,
}

impl PreviewExecutionReport {
    pub fn preview_execution_digest(&self) -> &str {
        &self.preview_execution_digest
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn preview_session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.preview_session_identity
    }

    pub fn lifecycle_state_kind(&self) -> BridgePreviewLifecycleStateKind {
        self.lifecycle_state_kind
    }

    pub fn execution_record_identity(&self) -> &PreviewExecutionRecordIdentity {
        &self.execution_record_identity
    }

    pub fn query_digest(&self) -> &ValidatedQueryDigest {
        &self.query_digest
    }

    pub fn result_digest(&self) -> &ResultDigest {
        &self.result_digest
    }

    pub fn comparison_eligibility_digest(&self) -> &str {
        &self.comparison_eligibility_digest
    }

    pub fn workflow_foundation_digest(&self) -> &str {
        &self.workflow_foundation_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewExecutionFailureClass {
    UnderlyingExecutionFailure,
    InvalidExecutionClass,
    InternalInvariantBreak,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewExecutionError {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    ExecutionFailure(ExecutionError),
    EvaluationClassMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    PreviewExecutionInvariantViolation {
        message: &'static str,
    },
}

impl PreviewExecutionError {
    pub fn failure_class(&self) -> PreviewExecutionFailureClass {
        match self {
            Self::ExecutionFailure(_) => PreviewExecutionFailureClass::UnderlyingExecutionFailure,
            Self::EvaluationClassMismatch { .. } => {
                PreviewExecutionFailureClass::InvalidExecutionClass
            }
            Self::PreviewExecutionInvariantViolation { .. } => {
                PreviewExecutionFailureClass::InternalInvariantBreak
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewExecutionEnvelope {
    pub(in crate::preview) binding: PreviewSessionPlanBinding,
    pub(in crate::preview) execution: ExecutionResultEnvelope,
    pub(super) comparison_eligibility: PreviewComparisonEligibilityArtifact,
    pub(super) workflow_foundation: AdmittedPreviewWorkflowFoundation,
    pub(super) report: PreviewExecutionReport,
    pub(super) counters: PreviewExecutionCounters,
}

impl PreviewExecutionEnvelope {
    pub fn binding(&self) -> &PreviewSessionPlanBinding {
        &self.binding
    }

    pub fn execution(&self) -> &ExecutionResultEnvelope {
        &self.execution
    }

    pub fn comparison_eligibility(&self) -> &PreviewComparisonEligibilityArtifact {
        &self.comparison_eligibility
    }

    pub fn workflow_foundation(&self) -> &PreviewWorkflowFoundationArtifact {
        self.workflow_foundation.artifact()
    }

    pub fn report(&self) -> &PreviewExecutionReport {
        &self.report
    }

    pub fn counters(&self) -> &PreviewExecutionCounters {
        &self.counters
    }

    pub fn check_invariants(&self) -> Result<(), PreviewExecutionError> {
        self.execution
            .check_invariants()
            .map_err(PreviewExecutionError::ExecutionFailure)?;

        if self
            .binding
            .report()
            .counters()
            .preview_lifecycle_rediscovery_count()
            != 0
        {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview lifecycle rediscovery must remain zero after execution",
            });
        }

        if self
            .binding
            .report()
            .counters()
            .preview_executor_rediscovery_count()
            != 0
        {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview executor rediscovery must remain zero after execution",
            });
        }

        if self
            .counters
            .binding_counters()
            .preview_lifecycle_rediscovery_count()
            != 0
            || self
                .counters
                .binding_counters()
                .preview_executor_rediscovery_count()
                != 0
        {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview execution counters must preserve zero rediscovery guarantees",
            });
        }

        if self
            .counters
            .execution_counters()
            .executor_semantic_rediscovery_count()
            != 0
        {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview execution cannot introduce executor semantic rediscovery",
            });
        }

        if self.counters.preview_execution_envelope_count() != 1 {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview execution must emit exactly one execution envelope",
            });
        }

        if self.counters.preview_execution_count() != 1 {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview execution must record exactly one preview execution",
            });
        }

        match self.binding.basis().binding_tuple().evaluation_class() {
            PreviewEvaluationClass::ReadOnly(_) => {
                if self.counters.preview_read_only_execution_count() != 1
                    || self.counters.preview_promotable_execution_count() != 0
                {
                    return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                        message: "read-only preview execution counters must remain class-explicit",
                    });
                }
            }
            PreviewEvaluationClass::PromotionEligible(_) => {
                if self.counters.preview_promotable_execution_count() != 1
                    || self.counters.preview_read_only_execution_count() != 0
                {
                    return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                        message:
                            "promotion-eligible preview execution counters must remain class-explicit",
                    });
                }
            }
        }

        if self.counters.preview_comparison_eligibility_proof_count() != 1 {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview execution must emit exactly one comparison eligibility proof",
            });
        }

        if self
            .counters
            .preview_workflow_foundation_artifact_lookup_count()
            != 1
        {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview execution must resolve exactly one workflow foundation artifact",
            });
        }

        if self.counters.preview_workflow_foundation_admission_count() != 1
            || self.counters.preview_workflow_foundation_denial_count() != 0
        {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message: "preview workflow foundation counters must remain admission-explicit",
            });
        }

        if self.counters.preview_work_avoided_by_explicit_basis_count() != 1 {
            return Err(PreviewExecutionError::PreviewExecutionInvariantViolation {
                message:
                    "preview execution must record exactly one explicit-basis work-avoided proof",
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadOnlyPreviewExecutionEnvelope {
    pub(super) inner: PreviewExecutionEnvelope,
}

impl ReadOnlyPreviewExecutionEnvelope {
    pub fn execution(&self) -> &ExecutionResultEnvelope {
        self.inner.execution()
    }

    pub fn basis(&self) -> &PreviewSessionBasis {
        self.inner.binding().basis()
    }

    pub fn lifecycle_metadata(&self) -> &PreviewLifecycleMetadata {
        self.inner.binding().lifecycle_metadata()
    }

    pub fn comparison_eligibility(&self) -> &PreviewComparisonEligibilityArtifact {
        self.inner.comparison_eligibility()
    }

    pub fn workflow_foundation(&self) -> &PreviewWorkflowFoundationArtifact {
        self.inner.workflow_foundation()
    }

    pub fn report(&self) -> &PreviewExecutionReport {
        self.inner.report()
    }

    pub fn counters(&self) -> &PreviewExecutionCounters {
        self.inner.counters()
    }

    #[cfg(test)]
    pub(crate) fn as_preview_execution(&self) -> &PreviewExecutionEnvelope {
        &self.inner
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionEligiblePreviewExecutionEnvelope {
    pub(super) inner: PreviewExecutionEnvelope,
}

impl PromotionEligiblePreviewExecutionEnvelope {
    pub fn execution(&self) -> &ExecutionResultEnvelope {
        self.inner.execution()
    }

    pub fn basis(&self) -> &PreviewSessionBasis {
        self.inner.binding().basis()
    }

    pub fn lifecycle_metadata(&self) -> &PreviewLifecycleMetadata {
        self.inner.binding().lifecycle_metadata()
    }

    pub fn comparison_eligibility(&self) -> &PreviewComparisonEligibilityArtifact {
        self.inner.comparison_eligibility()
    }

    pub fn workflow_foundation(&self) -> &PreviewWorkflowFoundationArtifact {
        self.inner.workflow_foundation()
    }

    pub fn report(&self) -> &PreviewExecutionReport {
        self.inner.report()
    }

    pub fn counters(&self) -> &PreviewExecutionCounters {
        self.inner.counters()
    }

    pub(crate) fn as_preview_execution(&self) -> &PreviewExecutionEnvelope {
        &self.inner
    }
}

impl From<IdentityDerivationDenial> for PreviewExecutionError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
