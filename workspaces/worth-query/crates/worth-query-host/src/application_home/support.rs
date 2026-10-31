//! One exhaustive directory support policy; operation extensions add their own families.
use worth_foundational::facade::DurableStateFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryFamilySupport {
    Supported,
    RefusedUntil(u8),
}

/// Base support does not admit later correction/component extensions.
pub const fn directory_family_support(family: DurableStateFamily) -> DirectoryFamilySupport {
    use DurableStateFamily::*;
    match family {
        ProgramActivation | PortablePackageMeaning => DirectoryFamilySupport::Supported,
        AggregateIdentity
        | AggregateMembership
        | AggregateLineage
        | AggregateProjectedCoordinates
        | RelationalPendingSettlement
        | WorldIdentityFrontiers
        | WorldProductBranches
        | WorldProductHeads
        | WorldCompositeHistory
        | WorldPerformedPublication
        | WorldAttemptProgress
        | WorldUnpublishedCustody
        | WorldComponentBranchCustody
        | QueryIdempotencyBinding
        | UnpublishedIdempotency
        | PendingApplicationPublication
        | ApplicationBranchCleanup
        | DispatchOutbox
        | AftermathCausality
        | CorrectionIntent
        | PublicationDiagnosticMaterial => DirectoryFamilySupport::Supported,
        WorldConditionalDefinitionCustody
        | SignalNodeDefinitions
        | SignalTopology
        | SignalBranchCells
        | SignalOwnerFrontiers
        | SignalBranchMetadata
        | SignalRuntimeDefinitions
        | BridgeConditionalDefinitions
        | BridgeSemanticRegistrations
        | BridgeCorrespondenceAllocation
        | BridgeOwnedAsyncDefinitions
        | BridgeDiagnosticRetention => DirectoryFamilySupport::RefusedUntil(5),
        ProgramSupportLifecycle
        | BranchAdoptionRecovery
        | StoppedBranchSetAdoption
        | OutputDemandRecords
        | OutputSourceCustody
        | PerformedSourcePreparation
        | ProgramOutputReadCustody => DirectoryFamilySupport::RefusedUntil(6),
        SignalOwnedAsyncCustody
        | SignalOwnedAsyncQueue
        | BridgeManagedTemporalIntents
        | BridgeOwnedAsyncCustody
        | PendingApplicationCleanup
        | ProducerCleanupCustody
        | ConditionalDecisions
        | RetainedConditionalCleanup
        | TemporalIdempotency
        | ResourceOccurrences
        | ResourceIngress
        | OwnedAsyncCustody
        | ManagedDirectRuns
        | ManagedWorkflowRuns
        | YieldedDirectRuns
        | YieldedWorkflowRuns
        | ProviderSuspensions
        | DirectYieldRecovery
        | WorkflowYieldRecovery
        | ManagedRestoreRecovery
        | DirectTerminalCleanup
        | WorkflowTerminalCleanup
        | WorkflowLineages
        | WorkflowDefinitions
        | WorkflowNodes
        | WorkflowConnections
        | WorkflowInstances
        | WorkflowTransitions
        | WorkflowProposals
        | WorkflowProposalCoverage
        | WorkflowAssessmentEvidence
        | WorkflowEvidenceDependencies
        | WorkflowApprovals
        | WorkflowOperationOutputs
        | WorkflowCapturedInputs
        | SourceOperationCustody
        | OperationCleanupCustody
        | BankManagedCleanup
        | HttpNativeCleanup
        | UiPresentationCustody
        | CapturedExternalInput
        | FixedOperationCustody
        | SignalResourceRequestCustody => DirectoryFamilySupport::RefusedUntil(7),
        DispatchAttempts
        | OutstandingDispatch
        | InboundCompletionRows
        | InboundMessageCustody
        | InstalledTransportCompletionCustody
        | TerminalAcknowledgement
        | InboundSourceControls
        | RecoveryClaims
        | RecoveryClaimDispositions
        | RecoveryHandleCustody
        | BankRecoveryClaim => DirectoryFamilySupport::RefusedUntil(8),
        CorrectionPreImages
        | CorrectionGovernedInput
        | CorrectionOriginalMeaning
        | OriginalOperationDescription
        | FixedDeliveryCustody
        | ServerSourceCustody
        | HttpOriginalOwners
        | UiStatusMutationOwner => DirectoryFamilySupport::RefusedUntil(9),
    }
}

/// Operation intent can require a later extension of an already-supported family.
pub(crate) fn directory_requirement_refusal(
    family: DurableStateFamily,
    required_phase: u8,
) -> Option<u8> {
    let family_phase = match directory_family_support(family) {
        DirectoryFamilySupport::Supported => 0,
        DirectoryFamilySupport::RefusedUntil(phase) => phase,
    };
    let phase = family_phase.max(required_phase);
    if phase <= 4 {
        None
    } else {
        Some(phase)
    }
}
