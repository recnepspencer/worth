use super::*;

mod stop_class;
mod stop_classify;

#[cfg(test)]
pub(crate) use stop_class::{
    WorthQueryRuntimeDeclarationFailureKind, WorthQueryRuntimeLookupFailureKind,
    WorthQueryRuntimeMissingArtifactKind,
};
pub use stop_class::{WorthQueryRuntimeMissingComponent, WorthQueryStopClass};

#[derive(Debug)]
#[non_exhaustive]
pub enum WorthQueryRuntimeError {
    MissingBackend,
    MissingRuntimeBridge,
    MissingSchemaAdapter,
    MissingSourceAdapter,
    MissingSnapshotIdentityAdapter,
    MissingWriteAuthority,
    MissingSignalSink,
    MissingSubscriptionActivation,
    MissingPreviewBasis,
    MissingInspectorEvidence,
    MissingIntentAuthority,
    ExistingTruthAssertionDenied(WorthQueryExistingTruthAssertionDenial),
    ExistingTruthProbeDenied(WorthQueryExistingTruthProbeDenial),
    MutationBindingDenied(WorthQueryExistingTruthBindingDenial),
    MutationContinuityDenied(WorthQueryContinuityMutationDenial),
    MutationContractDenied(crate::runtime::WorthQueryMutationContractDenial),
    GraphCompositionDenied(WorthQueryGraphCompositionDenial),
    MutationNamingDenied(WorthQueryNamingMutationDenial),
    MutationTargetReferenceDenied(WorthQuerySymbolicTargetReferenceDenial),
    ReadCompositionDenied(WorthQueryReadDenial),
    InstalledDomainAuthorityDenied(crate::domain_installation::WorthQueryDomainHandleDenial),
    Workspace(WorthQueryWorkspaceError),
    Program(WorthQueryProgramError),
    UnknownProgram(String),
    UnknownOperation {
        program_id: String,
        operation_id: String,
    },
    MissingLiveView(String),
    MissingLiveSubscription(String),
    MissingDerivedView(String),
    SharedReadStaleBasis {
        snapshot_identity: crate::memory_workspace::WorthQuerySnapshotIdentity,
    },
    JournalReplayDenied(WorthQueryJournalReplayDenial),
    MissingEffect(String),
    MissingPendingWriteIntent(String),
    RetainedRowDecode {
        view_name: String,
        stage: &'static str,
        message: String,
    },
    ComputedDeclaration {
        view_name: String,
        stage: &'static str,
        message: String,
    },
    EffectDeclaration {
        effect_name: String,
        stage: &'static str,
        message: String,
    },
    LiveSubscriptionInstallation {
        view_name: String,
        stage: &'static str,
        message: String,
    },
    UnsupportedAuthorityRequirement(WorthQueryAuthorityRequirement),
    ExistingTruthAssertionRequiresAuthorityLane {
        required_lane: WorthQueryAuthorityLane,
    },
    IntentCommitDenied {
        intent_name: String,
        stage: &'static str,
        message: String,
        evidence: WorthQueryIntentDenialEvidence,
    },
    IntentExecutionRoutingFailed {
        intent_name: String,
        stage: &'static str,
        message: String,
        evidence: WorthQueryIntentExecutionFailureEvidence,
        source: Box<WorthQueryRuntimeError>,
    },
    EffectPolicyDenied(WorthQueryEffectPolicyDenial),
    PreviewPromotionStaleBasis(WorthQueryPreviewPromotionDenialEvidence),
    PreviewPromotionAtomicBatchUnsupported(WorthQueryPreviewPromotionDenialEvidence),
    PreviewPromotionRebindingRequired(WorthQueryPreviewPromotionDenialEvidence),
    PreviewPromotionWriteFailed {
        evidence: WorthQueryPreviewPromotionDenialEvidence,
    },
    InvariantRegistration {
        stage: &'static str,
        message: String,
    },
    SessionLabelCollision {
        authority_lane: WorthQueryAuthorityLane,
        label: WorthQuerySessionLabel,
    },
    PreviewOperationEffectDenied {
        label: WorthQuerySessionLabel,
        stage: &'static str,
        message: String,
    },
    UnsupportedFacadeFamily(WorthQueryRuntimeSupportDenial),
}

impl std::fmt::Display for WorthQueryRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBackend => {
                write!(
                    f,
                    "Worth query runtime builder requires a backend; prefer runtime_bridge(...)+adapters+build_backend_from_parts() for ordinary bridge-backed runtimes, and use backend(...) only for explicit internal test or scaffold seams"
                )
            }
            Self::MissingRuntimeBridge => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires runtime_bridge(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingSchemaAdapter => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires schema_adapter(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingSourceAdapter => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires source_adapter(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingSnapshotIdentityAdapter => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires snapshot_identity(...); current snapshot truth must come from a typed backend authority before build_backend_from_parts()"
            ),
            Self::MissingWriteAuthority => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires write_authority(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingSignalSink => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires signal_sink(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingSubscriptionActivation => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires subscription_activation(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingPreviewBasis => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires preview_basis(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingInspectorEvidence => write!(
                f,
                "Worth query bridge-backed runtime bootstrap requires inspector_evidence(...); complete the builder-owned bridge-backed authority path before build_backend_from_parts()"
            ),
            Self::MissingIntentAuthority => write!(
                f,
                "Worth query runtime backend parts that claim intent support require an intent authority adapter"
            ),
            Self::ExistingTruthAssertionDenied(denial) => write!(f, "{denial}"),
            Self::ExistingTruthProbeDenied(denial) => write!(f, "{denial}"),
            Self::MutationBindingDenied(denial) => write!(f, "{denial}"),
            Self::MutationContinuityDenied(denial) => write!(f, "{denial}"),
            Self::MutationContractDenied(denial) => write!(f, "{denial}"),
            Self::GraphCompositionDenied(denial) => write!(f, "{denial}"),
            Self::MutationNamingDenied(denial) => write!(f, "{denial}"),
            Self::MutationTargetReferenceDenied(denial) => write!(f, "{denial}"),
            Self::ReadCompositionDenied(denial) => write!(f, "{denial}"),
            Self::InstalledDomainAuthorityDenied(denial) => write!(f, "{denial}"),
            Self::Workspace(error) => write!(f, "{error}"),
            Self::Program(error) => write!(f, "{error}"),
            Self::UnknownProgram(program) => write!(f, "unknown query program `{program}`"),
            Self::UnknownOperation {
                program_id,
                operation_id,
            } => write!(
                f,
                "unknown query operation `{operation_id}` in program `{program_id}`"
            ),
            Self::MissingLiveView(view) => write!(f, "unknown live view `{view}`"),
            Self::MissingLiveSubscription(view) => {
                write!(
                    f,
                    "live view `{view}` has no retained subscription installation"
                )
            }
            Self::MissingDerivedView(view) => write!(f, "unknown computed view `{view}`"),
            Self::SharedReadStaleBasis { snapshot_identity } => write!(
                f,
                "shared read basis `{}` is stale and can no longer serve published artifacts",
                snapshot_identity.terminal_projection_for_reporting()
            ),
            Self::JournalReplayDenied(denial) => write!(
                f,
                "journal replay denied: {} ({})",
                denial.kind().as_str(),
                denial.message()
            ),
            Self::MissingEffect(effect) => write!(f, "unknown effect `{effect}`"),
            Self::MissingPendingWriteIntent(effect) => {
                write!(f, "effect `{effect}` has no pending write intent delivery")
            }
            Self::RetainedRowDecode {
                view_name,
                stage,
                message,
            } => write!(
                f,
                "retained row decode for `{view_name}` failed during {stage}: {message}"
            ),
            Self::ComputedDeclaration {
                view_name,
                stage,
                message,
            } => write!(
                f,
                "computed declaration `{view_name}` failed during {stage}: {message}"
            ),
            Self::EffectDeclaration {
                effect_name,
                stage,
                message,
            } => write!(
                f,
                "effect declaration `{effect_name}` failed during {stage}: {message}"
            ),
            Self::LiveSubscriptionInstallation {
                view_name,
                stage,
                message,
            } => write!(
                f,
                "live view `{view_name}` subscription installation failed during {stage}: {message}"
            ),
            Self::UnsupportedAuthorityRequirement(requirement) => {
                write!(
                    f,
                    "authority requirement `{}` is not admitted by this runtime",
                    requirement.as_str()
                )
            }
            Self::ExistingTruthAssertionRequiresAuthorityLane { required_lane } => {
                write!(
                    f,
                    "existing-truth assertion currently requires the `{required_lane}` lane"
                )
            }
            Self::IntentCommitDenied {
                intent_name,
                stage,
                message,
                evidence: _,
            } => write!(
                f,
                "intent `{intent_name}` commit failed during {stage}: {message}"
            ),
            Self::IntentExecutionRoutingFailed {
                intent_name,
                stage,
                message,
                evidence: _,
                source: _,
            } => write!(
                f,
                "intent `{intent_name}` execution failed during {stage}: {message}"
            ),
            Self::EffectPolicyDenied(denial) => write!(f, "{denial}"),
            Self::PreviewPromotionStaleBasis(evidence) => write!(
                f,
                "preview promotion `{}` failed during {}: {}",
                evidence.label(),
                evidence.kind().as_str(),
                evidence.reason()
            ),
            Self::PreviewPromotionAtomicBatchUnsupported(evidence) => write!(
                f,
                "preview promotion `{}` failed during {}: {}",
                evidence.label(),
                evidence.kind().as_str(),
                evidence.reason()
            ),
            Self::PreviewPromotionRebindingRequired(evidence) => write!(
                f,
                "preview promotion `{}` failed during {}: {}",
                evidence.label(),
                evidence.kind().as_str(),
                evidence.reason()
            ),
            Self::PreviewPromotionWriteFailed { evidence } => write!(
                f,
                "preview promotion `{}` failed during {}: {}",
                evidence.label(),
                evidence.kind().as_str(),
                evidence.reason()
            ),
            Self::InvariantRegistration { stage, message } => write!(
                f,
                "runtime invariant registration failed during {stage}: {message}"
            ),
            Self::SessionLabelCollision {
                authority_lane,
                label,
            } => write!(
                f,
                "session label `{label}` is already admitted for `{}` authority lane",
                authority_lane.as_str()
            ),
            Self::PreviewOperationEffectDenied {
                label,
                stage,
                message,
            } => write!(
                f,
                "preview operation `{label}` failed during {stage}: {message}"
            ),
            Self::UnsupportedFacadeFamily(denial) => write!(f, "{denial}"),
        }
    }
}

impl std::error::Error for WorthQueryRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IntentExecutionRoutingFailed { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl WorthQueryRuntimeError {
    pub fn stop_class(&self) -> WorthQueryStopClass<'_> {
        stop_classify::classify_stop_class(self)
    }
}

impl From<WorthQueryWorkspaceError> for WorthQueryRuntimeError {
    fn from(value: WorthQueryWorkspaceError) -> Self {
        Self::Workspace(value)
    }
}

impl From<WorthQueryProgramError> for WorthQueryRuntimeError {
    fn from(value: WorthQueryProgramError) -> Self {
        Self::Program(value)
    }
}

impl From<worth_query_execution::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryRuntimeError
{
    fn from(denial: worth_query_execution::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::Workspace(denial.into())
    }
}
