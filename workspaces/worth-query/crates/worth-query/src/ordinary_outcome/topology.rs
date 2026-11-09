use crate::application::{
    WorthQueryDeclarationEntryOrchestrationRefusalClass,
    WorthQueryDeclarationEntryOrchestrationStage,
};
use crate::binding_pipeline::WorthQueryBindingLinkedArtifacts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOrdinaryBindingCheckedTopologyKind {
    Ambiguous,
    AspectConflict,
    AuthorityMismatch,
    BasisMismatch,
    ExplicitNarrowingRequired,
    MissingRequiredAspect,
    RebindRequired,
    Stale,
    Unavailable,
    Unsupported,
    WrongHandle,
    WrongWorld,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOrdinaryContinuationCheckedTopologyKind {
    Ambiguous,
    AsyncRequestDrift,
    AuthorityMismatch,
    BasisMismatch,
    Deferred,
    Denied,
    Failed,
    InstalledAuthorityDrift,
    LowerBindingMismatch,
    PreviewCrossedResidue,
    RemaskDrift,
    RebindRequired,
    ReplayDrift,
    Stale,
    StaleCompletion,
    Unavailable,
    Unsupported,
    WrongHandle,
    WrongWorld,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind {
    Ambiguous,
    AspectConflict,
    AuthorityMismatch,
    BasisMismatch,
    Deferred,
    Denied,
    Failed,
    InstalledAuthorityDrift,
    MissingRequiredAspect,
    RebindRequired,
    Stale,
    Unavailable,
    Unsupported,
    WrongHandle,
    WrongWorld,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOrdinaryContributionComposedCheckedTopologyKind {
    Deferred,
    DeclarationDenied,
    ContributionDenied,
    Stale,
    RebindRequired,
    Unsupported,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorthQueryOrdinaryCheckedTopologyRepr {
    EvidenceIdentityDerivationDenied(
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    Orchestration {
        stop_stage: WorthQueryDeclarationEntryOrchestrationStage,
        retained_digest: Option<String>,
        refusal_class: Option<WorthQueryDeclarationEntryOrchestrationRefusalClass>,
    },
    Binding {
        kind: WorthQueryOrdinaryBindingCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
    },
    Continuation {
        kind: WorthQueryOrdinaryContinuationCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
    },
    SignalCompatibilityOrchestration {
        kind: WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
    },
    ContributionComposed {
        kind: WorthQueryOrdinaryContributionComposedCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
        contribution_digest: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOrdinaryCheckedTopology {
    repr: WorthQueryOrdinaryCheckedTopologyRepr,
}

impl WorthQueryOrdinaryCheckedTopology {
    pub fn identity_refused(
        cause: crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    ) -> Self {
        Self {
            repr: WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(cause),
        }
    }
    pub fn identity_derivation_denial(
        &self,
    ) -> Option<&crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(cause) => {
                Some(cause)
            }
            _ => None,
        }
    }

    pub fn orchestration(
        stop_stage: WorthQueryDeclarationEntryOrchestrationStage,
        retained_digest: Option<String>,
        refusal_class: Option<WorthQueryDeclarationEntryOrchestrationRefusalClass>,
    ) -> Self {
        Self {
            repr: WorthQueryOrdinaryCheckedTopologyRepr::Orchestration {
                stop_stage,
                retained_digest,
                refusal_class,
            },
        }
    }

    pub(crate) fn binding(
        kind: WorthQueryOrdinaryBindingCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
    ) -> Self {
        Self {
            repr: WorthQueryOrdinaryCheckedTopologyRepr::Binding {
                kind,
                linked_artifacts,
            },
        }
    }

    pub(crate) fn continuation(
        kind: WorthQueryOrdinaryContinuationCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
    ) -> Self {
        Self {
            repr: WorthQueryOrdinaryCheckedTopologyRepr::Continuation {
                kind,
                linked_artifacts,
            },
        }
    }

    pub(crate) fn signal_compatibility_orchestration(
        kind: WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
    ) -> Self {
        Self {
            repr: WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration {
                kind,
                linked_artifacts,
            },
        }
    }

    pub(crate) fn contribution_composed(
        kind: WorthQueryOrdinaryContributionComposedCheckedTopologyKind,
        linked_artifacts: WorthQueryBindingLinkedArtifacts,
        contribution_digest: Option<String>,
    ) -> Self {
        Self {
            repr: WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed {
                kind,
                linked_artifacts,
                contribution_digest,
            },
        }
    }

    pub fn orchestration_stop_stage(&self) -> Option<WorthQueryDeclarationEntryOrchestrationStage> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { stop_stage, .. } => {
                Some(*stop_stage)
            }
            WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn orchestration_retained_digest(&self) -> Option<&str> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration {
                retained_digest, ..
            } => retained_digest.as_deref(),
            WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn orchestration_refusal_class(
        &self,
    ) -> Option<WorthQueryDeclarationEntryOrchestrationRefusalClass> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { refusal_class, .. } => {
                *refusal_class
            }
            WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn binding_kind(&self) -> Option<WorthQueryOrdinaryBindingCheckedTopologyKind> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::Binding { kind, .. } => Some(*kind),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn binding_linked_artifacts(&self) -> Option<&WorthQueryBindingLinkedArtifacts> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::Binding {
                linked_artifacts, ..
            } => Some(linked_artifacts),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn continuation_kind(&self) -> Option<WorthQueryOrdinaryContinuationCheckedTopologyKind> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::Continuation { kind, .. } => Some(*kind),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn continuation_linked_artifacts(&self) -> Option<&WorthQueryBindingLinkedArtifacts> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::Continuation {
                linked_artifacts, ..
            } => Some(linked_artifacts),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn signal_compatibility_orchestration_kind(
        &self,
    ) -> Option<WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration {
                kind,
                ..
            } => Some(*kind),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn signal_compatibility_orchestration_linked_artifacts(
        &self,
    ) -> Option<&WorthQueryBindingLinkedArtifacts> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration {
                linked_artifacts,
                ..
            } => Some(linked_artifacts),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { .. } => None,
        }
    }

    pub fn contribution_composed_kind(
        &self,
    ) -> Option<WorthQueryOrdinaryContributionComposedCheckedTopologyKind> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed { kind, .. } => Some(*kind),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. } => {
                None
            }
        }
    }

    pub fn contribution_composed_linked_artifacts(
        &self,
    ) -> Option<&WorthQueryBindingLinkedArtifacts> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed {
                linked_artifacts,
                ..
            } => Some(linked_artifacts),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. } => {
                None
            }
        }
    }

    pub fn contribution_composed_digest(&self) -> Option<&str> {
        match &self.repr {
            WorthQueryOrdinaryCheckedTopologyRepr::EvidenceIdentityDerivationDenied(_) => None,
            WorthQueryOrdinaryCheckedTopologyRepr::ContributionComposed {
                contribution_digest,
                ..
            } => contribution_digest.as_deref(),
            WorthQueryOrdinaryCheckedTopologyRepr::Orchestration { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Binding { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::Continuation { .. }
            | WorthQueryOrdinaryCheckedTopologyRepr::SignalCompatibilityOrchestration { .. } => {
                None
            }
        }
    }
}
