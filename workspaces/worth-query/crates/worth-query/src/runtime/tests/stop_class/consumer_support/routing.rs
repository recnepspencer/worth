use super::super::super::support::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in super::super) enum ConsumerStopRoute {
    Advancement(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    ForeignPhase(
        worth_query_execution::facade::application_contribution::WorthQueryForeignAdvancementPhase,
    ),
    InstalledDomainAuthorityDenied(crate::domain_installation::WorthQueryDomainHandleDenialKind),
    MissingRuntimeComponent(WorthQueryRuntimeMissingComponent),
    ExistingTruthAssertionDenied(WorthQueryExistingTruthAssertionDenialKind),
    ExistingTruthProbeDenied(WorthQueryExistingTruthProbeDenialKind),
    MutationBindingDenied(WorthQueryExistingTruthBindingDenialKind),
    MutationContinuityDenied(WorthQueryContinuityMutationDenialKind),
    MutationContractDenied(WorthQueryMutationContractDenialKind),
    GraphCompositionDenied(WorthQueryGraphCompositionDenialKind),
    MutationNamingDenied(WorthQueryNamingMutationDenialKind),
    MutationTargetReferenceDenied(WorthQuerySymbolicTargetReferenceDenialKind),
    ReadCompositionDenied(WorthQueryReadDenialKind),
    WorkspaceDenied,
    ProgramDenied,
    RuntimeLookupDenied(WorthQueryRuntimeLookupFailureKind),
    MissingRuntimeArtifact(WorthQueryRuntimeMissingArtifactKind),
    SharedReadStaleBasis,
    JournalReplayDenied(WorthQueryJournalReplayDenialKind),
    RuntimeDeclarationDenied(WorthQueryRuntimeDeclarationFailureKind),
    PreviewOperationEffectDenied(crate::WorthQueryEvidenceIdentity),
    UnsupportedAuthorityRequirement(WorthQueryAuthorityRequirement),
    ExistingTruthAssertionRequiresAuthorityLane(WorthQueryAuthorityLane),
    IntentCommitDenied,
    IntentExecutionRoutingFailed(WorthQueryRuntimeDeclarationFailureKind),
    EffectPolicyDenied,
    PreviewPromotionDenied(WorthQueryPreviewPromotionDenialKind),
    FamilyAdmissionDenied {
        family: WorthQueryRuntimeFacadeFamily,
        status: WorthQueryRuntimeFamilySupportStatus,
        teaching_posture: Option<WorthQueryRuntimeFamilyTeachingPosture>,
    },
    SessionLabelCollision(WorthQueryAuthorityLane),
}

pub(in super::super) fn route_consumer_stop_class(
    error: &WorthQueryRuntimeError,
) -> ConsumerStopRoute {
    match error.stop_class() {
        WorthQueryStopClass::Advancement { denial } => ConsumerStopRoute::Advancement(*denial),
        WorthQueryStopClass::ForeignPhase { denial } => ConsumerStopRoute::ForeignPhase(*denial),
        WorthQueryStopClass::InstalledDomainAuthorityDenied { denial } => {
            ConsumerStopRoute::InstalledDomainAuthorityDenied(denial.kind())
        }
        WorthQueryStopClass::MissingRuntimeComponent { component } => {
            ConsumerStopRoute::MissingRuntimeComponent(component)
        }
        WorthQueryStopClass::ExistingTruthAssertionDenied { denial } => {
            ConsumerStopRoute::ExistingTruthAssertionDenied(denial.kind())
        }
        WorthQueryStopClass::ExistingTruthProbeDenied { denial } => {
            ConsumerStopRoute::ExistingTruthProbeDenied(denial.kind())
        }
        WorthQueryStopClass::MutationBindingDenied { denial } => {
            ConsumerStopRoute::MutationBindingDenied(denial.kind())
        }
        WorthQueryStopClass::MutationContinuityDenied { denial } => {
            ConsumerStopRoute::MutationContinuityDenied(denial.kind())
        }
        WorthQueryStopClass::MutationContractDenied { denial } => {
            ConsumerStopRoute::MutationContractDenied(denial.kind())
        }
        WorthQueryStopClass::GraphCompositionDenied { denial } => {
            ConsumerStopRoute::GraphCompositionDenied(denial.kind())
        }
        WorthQueryStopClass::MutationNamingDenied { denial } => {
            ConsumerStopRoute::MutationNamingDenied(denial.kind())
        }
        WorthQueryStopClass::MutationTargetReferenceDenied { denial } => {
            ConsumerStopRoute::MutationTargetReferenceDenied(denial.kind())
        }
        WorthQueryStopClass::ReadCompositionDenied { denial } => {
            ConsumerStopRoute::ReadCompositionDenied(denial.kind().clone())
        }
        WorthQueryStopClass::Workspace { .. } => ConsumerStopRoute::WorkspaceDenied,
        WorthQueryStopClass::Program { .. } => ConsumerStopRoute::ProgramDenied,
        WorthQueryStopClass::RuntimeLookupFailed { kind, .. } => {
            ConsumerStopRoute::RuntimeLookupDenied(kind)
        }
        WorthQueryStopClass::MissingRuntimeArtifact { kind, .. } => {
            ConsumerStopRoute::MissingRuntimeArtifact(kind)
        }
        WorthQueryStopClass::SharedReadStaleBasis { .. } => ConsumerStopRoute::SharedReadStaleBasis,
        WorthQueryStopClass::JournalReplayDenied { denial } => {
            ConsumerStopRoute::JournalReplayDenied(denial.kind())
        }
        WorthQueryStopClass::RuntimeDeclarationFailed { kind, .. } => {
            ConsumerStopRoute::RuntimeDeclarationDenied(kind)
        }
        WorthQueryStopClass::PreviewOperationEffectDenied { label, .. } => {
            ConsumerStopRoute::PreviewOperationEffectDenied(label.identity_digest().clone())
        }
        WorthQueryStopClass::UnsupportedAuthorityRequirement { requirement } => {
            ConsumerStopRoute::UnsupportedAuthorityRequirement(requirement.clone())
        }
        WorthQueryStopClass::ExistingTruthAssertionRequiresAuthorityLane { required_lane } => {
            ConsumerStopRoute::ExistingTruthAssertionRequiresAuthorityLane(required_lane)
        }
        WorthQueryStopClass::IntentCommitDenied { .. } => ConsumerStopRoute::IntentCommitDenied,
        WorthQueryStopClass::IntentExecutionRoutingFailed { source, .. } => {
            match source.stop_class() {
                WorthQueryStopClass::RuntimeDeclarationFailed { kind, .. } => {
                    ConsumerStopRoute::IntentExecutionRoutingFailed(kind)
                }
                other => panic!("consumer source route expected declaration stop, got {other:?}"),
            }
        }
        WorthQueryStopClass::EffectPolicyDenied { .. } => ConsumerStopRoute::EffectPolicyDenied,
        WorthQueryStopClass::PreviewPromotionDenied { kind, .. } => {
            ConsumerStopRoute::PreviewPromotionDenied(kind)
        }
        WorthQueryStopClass::FamilyAdmissionDenied {
            family,
            status,
            teaching_posture,
            ..
        } => ConsumerStopRoute::FamilyAdmissionDenied {
            family,
            status,
            teaching_posture,
        },
        WorthQueryStopClass::SessionLabelCollision { authority_lane, .. } => {
            ConsumerStopRoute::SessionLabelCollision(authority_lane)
        }
    }
}
