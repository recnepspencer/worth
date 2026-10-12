use super::stop_class::{
    WorthQueryRuntimeDeclarationFailureKind, WorthQueryRuntimeLookupFailureKind,
    WorthQueryRuntimeMissingArtifactKind, WorthQueryRuntimeMissingComponent, WorthQueryStopClass,
};
use super::*;

pub(super) fn classify_stop_class(error: &WorthQueryRuntimeError) -> WorthQueryStopClass<'_> {
    match error {
        WorthQueryRuntimeError::Advancement(denial) => WorthQueryStopClass::Advancement { denial },
        WorthQueryRuntimeError::ForeignPhase(denial) => {
            WorthQueryStopClass::ForeignPhase { denial }
        }
        WorthQueryRuntimeError::InstalledDomainAuthorityDenied(denial) => {
            WorthQueryStopClass::InstalledDomainAuthorityDenied { denial }
        }
        WorthQueryRuntimeError::MissingBackend => WorthQueryStopClass::MissingRuntimeComponent {
            component: WorthQueryRuntimeMissingComponent::Backend,
        },
        WorthQueryRuntimeError::MissingRuntimeBridge => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::RuntimeBridge,
            }
        }
        WorthQueryRuntimeError::MissingSchemaAdapter => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::SchemaAdapter,
            }
        }
        WorthQueryRuntimeError::MissingSourceAdapter => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::SourceAdapter,
            }
        }
        WorthQueryRuntimeError::MissingSnapshotIdentityAdapter => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::SnapshotIdentityAdapter,
            }
        }
        WorthQueryRuntimeError::MissingWriteAuthority => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::WriteAuthority,
            }
        }
        WorthQueryRuntimeError::MissingSignalSink => WorthQueryStopClass::MissingRuntimeComponent {
            component: WorthQueryRuntimeMissingComponent::SignalSink,
        },
        WorthQueryRuntimeError::MissingSubscriptionActivation => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::SubscriptionActivation,
            }
        }
        WorthQueryRuntimeError::MissingPreviewBasis => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::PreviewBasis,
            }
        }
        WorthQueryRuntimeError::MissingInspectorEvidence => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::InspectorEvidence,
            }
        }
        WorthQueryRuntimeError::MissingIntentAuthority => {
            WorthQueryStopClass::MissingRuntimeComponent {
                component: WorthQueryRuntimeMissingComponent::IntentAuthority,
            }
        }
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial) => {
            WorthQueryStopClass::ExistingTruthAssertionDenied { denial }
        }
        WorthQueryRuntimeError::ExistingTruthProbeDenied(denial) => {
            WorthQueryStopClass::ExistingTruthProbeDenied { denial }
        }
        WorthQueryRuntimeError::MutationBindingDenied(denial) => {
            WorthQueryStopClass::MutationBindingDenied { denial }
        }
        WorthQueryRuntimeError::MutationContinuityDenied(denial) => {
            WorthQueryStopClass::MutationContinuityDenied { denial }
        }
        WorthQueryRuntimeError::MutationContractDenied(denial) => {
            WorthQueryStopClass::MutationContractDenied { denial }
        }
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            WorthQueryStopClass::GraphCompositionDenied { denial }
        }
        WorthQueryRuntimeError::MutationNamingDenied(denial) => {
            WorthQueryStopClass::MutationNamingDenied { denial }
        }
        WorthQueryRuntimeError::MutationTargetReferenceDenied(denial) => {
            WorthQueryStopClass::MutationTargetReferenceDenied { denial }
        }
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            WorthQueryStopClass::ReadCompositionDenied { denial }
        }
        WorthQueryRuntimeError::Workspace(inner) => WorthQueryStopClass::Workspace { error: inner },
        WorthQueryRuntimeError::Program(inner) => WorthQueryStopClass::Program { error: inner },
        WorthQueryRuntimeError::UnknownProgram(program_id) => {
            WorthQueryStopClass::RuntimeLookupFailed {
                kind: WorthQueryRuntimeLookupFailureKind::UnknownProgram,
                program_id,
                operation_id: None,
            }
        }
        WorthQueryRuntimeError::UnknownOperation {
            program_id,
            operation_id,
        } => WorthQueryStopClass::RuntimeLookupFailed {
            kind: WorthQueryRuntimeLookupFailureKind::UnknownOperation,
            program_id,
            operation_id: Some(operation_id),
        },
        WorthQueryRuntimeError::MissingLiveView(view_name) => {
            WorthQueryStopClass::MissingRuntimeArtifact {
                kind: WorthQueryRuntimeMissingArtifactKind::LiveView,
                name: view_name,
            }
        }
        WorthQueryRuntimeError::MissingLiveSubscription(view_name) => {
            WorthQueryStopClass::MissingRuntimeArtifact {
                kind: WorthQueryRuntimeMissingArtifactKind::LiveSubscription,
                name: view_name,
            }
        }
        WorthQueryRuntimeError::MissingDerivedView(view_name) => {
            WorthQueryStopClass::MissingRuntimeArtifact {
                kind: WorthQueryRuntimeMissingArtifactKind::DerivedView,
                name: view_name,
            }
        }
        WorthQueryRuntimeError::SharedReadStaleBasis { snapshot_identity } => {
            WorthQueryStopClass::SharedReadStaleBasis { snapshot_identity }
        }
        WorthQueryRuntimeError::JournalReplayDenied(denial) => {
            WorthQueryStopClass::JournalReplayDenied { denial }
        }
        WorthQueryRuntimeError::MissingEffect(effect_name) => {
            WorthQueryStopClass::MissingRuntimeArtifact {
                kind: WorthQueryRuntimeMissingArtifactKind::Effect,
                name: effect_name,
            }
        }
        WorthQueryRuntimeError::MissingPendingWriteIntent(effect_name) => {
            WorthQueryStopClass::MissingRuntimeArtifact {
                kind: WorthQueryRuntimeMissingArtifactKind::PendingWriteIntent,
                name: effect_name,
            }
        }
        WorthQueryRuntimeError::RetainedRowDecode {
            view_name,
            stage,
            message,
        } => WorthQueryStopClass::RuntimeDeclarationFailed {
            kind: WorthQueryRuntimeDeclarationFailureKind::RetainedRowDecode,
            name: view_name,
            stage,
            message,
        },
        WorthQueryRuntimeError::ComputedDeclaration {
            view_name,
            stage,
            message,
        } => WorthQueryStopClass::RuntimeDeclarationFailed {
            kind: WorthQueryRuntimeDeclarationFailureKind::ComputedDeclaration,
            name: view_name,
            stage,
            message,
        },
        WorthQueryRuntimeError::EffectDeclaration {
            effect_name,
            stage,
            message,
        } => WorthQueryStopClass::RuntimeDeclarationFailed {
            kind: WorthQueryRuntimeDeclarationFailureKind::EffectDeclaration,
            name: effect_name,
            stage,
            message,
        },
        WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name,
            stage,
            message,
        } => WorthQueryStopClass::RuntimeDeclarationFailed {
            kind: WorthQueryRuntimeDeclarationFailureKind::LiveSubscriptionInstallation,
            name: view_name,
            stage,
            message,
        },
        WorthQueryRuntimeError::SessionLabelCollision {
            authority_lane,
            label,
        } => WorthQueryStopClass::SessionLabelCollision {
            authority_lane: *authority_lane,
            label,
        },
        WorthQueryRuntimeError::UnsupportedAuthorityRequirement(requirement) => {
            WorthQueryStopClass::UnsupportedAuthorityRequirement { requirement }
        }
        WorthQueryRuntimeError::ExistingTruthAssertionRequiresAuthorityLane { required_lane } => {
            WorthQueryStopClass::ExistingTruthAssertionRequiresAuthorityLane {
                required_lane: *required_lane,
            }
        }
        WorthQueryRuntimeError::IntentCommitDenied {
            intent_name,
            stage,
            message,
            evidence,
        } => WorthQueryStopClass::IntentCommitDenied {
            intent_name,
            stage,
            message,
            evidence,
        },
        WorthQueryRuntimeError::IntentExecutionRoutingFailed {
            intent_name,
            stage,
            message,
            evidence,
            source,
        } => WorthQueryStopClass::IntentExecutionRoutingFailed {
            intent_name,
            stage,
            message,
            evidence,
            source,
        },
        WorthQueryRuntimeError::EffectPolicyDenied(denial) => {
            WorthQueryStopClass::EffectPolicyDenied { denial: *denial }
        }
        WorthQueryRuntimeError::PreviewPromotionStaleBasis(evidence) => {
            WorthQueryStopClass::PreviewPromotionDenied {
                kind: WorthQueryPreviewPromotionDenialKind::StaleBasis,
                evidence,
            }
        }
        WorthQueryRuntimeError::PreviewPromotionAtomicBatchUnsupported(evidence) => {
            WorthQueryStopClass::PreviewPromotionDenied {
                kind: WorthQueryPreviewPromotionDenialKind::AtomicBatchUnsupported,
                evidence,
            }
        }
        WorthQueryRuntimeError::PreviewPromotionRebindingRequired(evidence) => {
            WorthQueryStopClass::PreviewPromotionDenied {
                kind: WorthQueryPreviewPromotionDenialKind::RebindingRequired,
                evidence,
            }
        }
        WorthQueryRuntimeError::PreviewPromotionWriteFailed { evidence } => {
            WorthQueryStopClass::PreviewPromotionDenied {
                kind: WorthQueryPreviewPromotionDenialKind::WriteFailed,
                evidence,
            }
        }
        WorthQueryRuntimeError::InvariantRegistration { stage, message } => {
            WorthQueryStopClass::RuntimeDeclarationFailed {
                kind: WorthQueryRuntimeDeclarationFailureKind::InvariantRegistration,
                name: "runtime-invariant-registration",
                stage,
                message,
            }
        }
        WorthQueryRuntimeError::PreviewOperationEffectDenied {
            label,
            stage,
            message,
        } => WorthQueryStopClass::PreviewOperationEffectDenied {
            label,
            stage,
            message,
        },
        WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
            WorthQueryStopClass::FamilyAdmissionDenied {
                family: denial.family(),
                status: denial.status(),
                teaching_posture: denial.teaching_posture(),
                reason: denial.reason(),
            }
        }
    }
}
