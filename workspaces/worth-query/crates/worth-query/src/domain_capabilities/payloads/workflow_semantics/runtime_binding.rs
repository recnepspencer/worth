use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use worth_runtime_bridge::facade::BridgePreviewSessionIdentity;

use crate::basis::ExecutionPreflightBundle;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::workflow::WorkflowPreviewEvaluationClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowRuntimeBindingSemantics {
    RuntimePreflight {
        runtime_snapshot_identity: WorthQuerySnapshotIdentity,
    },
    RuntimePreflightBundle {
        preflight: ExecutionPreflightBundle,
    },
    PreviewFoundation {
        preview_session_identity: BridgePreviewSessionIdentity,
        evaluation_class: WorkflowPreviewEvaluationClass,
    },
}

impl WorthQueryWorkflowRuntimeBindingSemantics {
    pub fn runtime_preflight_snapshot_identity(
        runtime_snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Self {
        Self::RuntimePreflight {
            runtime_snapshot_identity,
        }
    }

    pub fn runtime_preflight_bundle(preflight: ExecutionPreflightBundle) -> Self {
        Self::RuntimePreflightBundle { preflight }
    }

    pub fn preview_foundation(
        preview_session_identity: BridgePreviewSessionIdentity,
        evaluation_class: WorkflowPreviewEvaluationClass,
    ) -> Self {
        Self::PreviewFoundation {
            preview_session_identity,
            evaluation_class,
        }
    }

    pub fn runtime_snapshot_identity(&self) -> Option<WorthQuerySnapshotIdentity> {
        match self {
            Self::RuntimePreflight {
                runtime_snapshot_identity,
            } => Some(runtime_snapshot_identity.clone()),
            Self::RuntimePreflightBundle { preflight } => {
                Some(WorthQuerySnapshotIdentity::preview(
                    preflight.basis().identity().snapshot_identity().clone(),
                ))
            }
            Self::PreviewFoundation { .. } => None,
        }
    }

    pub fn runtime_preflight_bundle_ref(&self) -> Option<&ExecutionPreflightBundle> {
        match self {
            Self::RuntimePreflightBundle { preflight } => Some(preflight),
            Self::RuntimePreflight { .. } | Self::PreviewFoundation { .. } => None,
        }
    }

    pub fn preview_foundation_binding(
        &self,
    ) -> Option<(
        &BridgePreviewSessionIdentity,
        WorkflowPreviewEvaluationClass,
    )> {
        match self {
            Self::PreviewFoundation {
                preview_session_identity,
                evaluation_class,
            } => Some((preview_session_identity, evaluation_class.clone())),
            Self::RuntimePreflight { .. } | Self::RuntimePreflightBundle { .. } => None,
        }
    }

    pub(crate) fn semantics_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(match self {
            Self::RuntimePreflight {
                runtime_snapshot_identity,
            } => domain_capability_scope_encoder("worth_query_workflow_runtime_binding_v1")
                .field_shape(WorthQueryEvidenceTag::new("kind"), "runtime_preflight")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("runtime_snapshot"),
                    &runtime_snapshot_identity.evidence_identity(),
                )
                .seal()?,
            Self::RuntimePreflightBundle { preflight } => {
                domain_capability_scope_encoder("worth_query_workflow_runtime_binding_v1")
                    .field_shape(
                        WorthQueryEvidenceTag::new("kind"),
                        "runtime_preflight_bundle",
                    )
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("plan"),
                        &preflight.plan().query().plan_digest().evidence_identity(),
                    )
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("canonical_query"),
                        &crate::identity::canonical_query_evidence_identity(
                            preflight.plan().query().canonical_query_digest(),
                        ),
                    )
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("basis_proof"),
                        &preflight.basis().proof().digest().evidence_identity(),
                    )
                    .seal()?
            }
            Self::PreviewFoundation {
                preview_session_identity,
                evaluation_class,
            } => domain_capability_scope_encoder("worth_query_workflow_runtime_binding_v1")
                .field_shape(WorthQueryEvidenceTag::new("kind"), "preview_foundation")
                .field_bridge_authority_identity(
                    WorthQueryEvidenceTag::new("preview_session"),
                    &preview_session_identity.bridge_trust_boundary(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("evaluation_class"),
                    evaluation_class.as_str(),
                )
                .seal()?,
        })
    }
}
