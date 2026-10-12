use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeMissingComponent {
    Backend,
    RuntimeBridge,
    SchemaAdapter,
    SourceAdapter,
    SnapshotIdentityAdapter,
    WriteAuthority,
    SignalSink,
    SubscriptionActivation,
    PreviewBasis,
    InspectorEvidence,
    IntentAuthority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeLookupFailureKind {
    UnknownProgram,
    UnknownOperation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeMissingArtifactKind {
    LiveView,
    LiveSubscription,
    DerivedView,
    Effect,
    PendingWriteIntent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeDeclarationFailureKind {
    RetainedRowDecode,
    ComputedDeclaration,
    EffectDeclaration,
    LiveSubscriptionInstallation,
    InvariantRegistration,
}

#[derive(Clone, Copy, Debug)]
pub enum WorthQueryStopClass<'a> {
    Advancement { denial: &'a worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial },
    ForeignPhase { denial: &'a worth_query_execution::facade::application_contribution::WorthQueryForeignAdvancementPhase },
    InstalledDomainAuthorityDenied {
        denial: &'a crate::domain_installation::WorthQueryDomainHandleDenial,
    },
    MissingRuntimeComponent {
        component: WorthQueryRuntimeMissingComponent,
    },
    ExistingTruthAssertionDenied {
        denial: &'a WorthQueryExistingTruthAssertionDenial,
    },
    ExistingTruthProbeDenied {
        denial: &'a WorthQueryExistingTruthProbeDenial,
    },
    MutationBindingDenied {
        denial: &'a WorthQueryExistingTruthBindingDenial,
    },
    MutationContinuityDenied {
        denial: &'a WorthQueryContinuityMutationDenial,
    },
    MutationContractDenied {
        denial: &'a crate::runtime::WorthQueryMutationContractDenial,
    },
    GraphCompositionDenied {
        denial: &'a WorthQueryGraphCompositionDenial,
    },
    MutationNamingDenied {
        denial: &'a WorthQueryNamingMutationDenial,
    },
    MutationTargetReferenceDenied {
        denial: &'a WorthQuerySymbolicTargetReferenceDenial,
    },
    ReadCompositionDenied {
        denial: &'a WorthQueryReadDenial,
    },
    Workspace {
        error: &'a WorthQueryWorkspaceError,
    },
    Program {
        error: &'a WorthQueryProgramError,
    },
    RuntimeLookupFailed {
        kind: WorthQueryRuntimeLookupFailureKind,
        program_id: &'a str,
        operation_id: Option<&'a str>,
    },
    MissingRuntimeArtifact {
        kind: WorthQueryRuntimeMissingArtifactKind,
        name: &'a str,
    },
    SharedReadStaleBasis {
        snapshot_identity: &'a crate::memory_workspace::WorthQuerySnapshotIdentity,
    },
    JournalReplayDenied {
        denial: &'a WorthQueryJournalReplayDenial,
    },
    RuntimeDeclarationFailed {
        kind: WorthQueryRuntimeDeclarationFailureKind,
        name: &'a str,
        stage: &'static str,
        message: &'a str,
    },
    PreviewOperationEffectDenied {
        label: &'a WorthQuerySessionLabel,
        stage: &'static str,
        message: &'a str,
    },
    SessionLabelCollision {
        authority_lane: WorthQueryAuthorityLane,
        label: &'a WorthQuerySessionLabel,
    },
    UnsupportedAuthorityRequirement {
        requirement: &'a WorthQueryAuthorityRequirement,
    },
    ExistingTruthAssertionRequiresAuthorityLane {
        required_lane: WorthQueryAuthorityLane,
    },
    IntentCommitDenied {
        intent_name: &'a str,
        stage: &'static str,
        message: &'a str,
        evidence: &'a WorthQueryIntentDenialEvidence,
    },
    IntentExecutionRoutingFailed {
        intent_name: &'a str,
        stage: &'static str,
        message: &'a str,
        evidence: &'a WorthQueryIntentExecutionFailureEvidence,
        source: &'a WorthQueryRuntimeError,
    },
    EffectPolicyDenied {
        denial: WorthQueryEffectPolicyDenial,
    },
    PreviewPromotionDenied {
        kind: WorthQueryPreviewPromotionDenialKind,
        evidence: &'a WorthQueryPreviewPromotionDenialEvidence,
    },
    FamilyAdmissionDenied {
        family: WorthQueryRuntimeFacadeFamily,
        status: WorthQueryRuntimeFamilySupportStatus,
        teaching_posture: Option<WorthQueryRuntimeFamilyTeachingPosture>,
        reason: &'a str,
    },
}
