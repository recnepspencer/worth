//! One exhaustive directory support policy; operation extensions add their own families.
use worth_foundational::facade::DurableStateFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryFamilySupport {
    Supported,
    RefusedUntil(u8),
}

/// The phase whose dispatch-custody extension Directory Homes now deliver.
const DISPATCH_CUSTODY_PHASE: u8 = 8;

/// The families that extension is delivered for: an emission's outbox row and
/// its attempt journal. No other family gains a phase-8 extension here.
fn delivers_dispatch_custody(family: DurableStateFamily) -> bool {
    family == DurableStateFamily::DispatchOutbox || family == DurableStateFamily::DispatchAttempts
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
        | DispatchAttempts
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
        OutstandingDispatch
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
///
/// Phase 8's external dispatch custody is delivered for exactly the outbox
/// row and the attempt journal. An emission is therefore admitted; a Home
/// that has dispatched cannot yet close gracefully (its receipt-basis owner
/// refuses). Inbound completion and recovery claims stay refused through
/// their own families.
pub(crate) fn directory_requirement_refusal(
    family: DurableStateFamily,
    required_phase: u8,
) -> Option<u8> {
    let family_phase = match directory_family_support(family) {
        DirectoryFamilySupport::Supported => 0,
        DirectoryFamilySupport::RefusedUntil(phase) => phase,
    };
    let phase = family_phase.max(required_phase);
    let dispatch_custody = family_phase == 0
        && required_phase == DISPATCH_CUSTODY_PHASE
        && delivers_dispatch_custody(family);
    if phase <= 4 || dispatch_custody {
        None
    } else {
        Some(phase)
    }
}
