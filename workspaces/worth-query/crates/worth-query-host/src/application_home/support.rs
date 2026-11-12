//! One exhaustive directory support policy; operation extensions add their own families.
use worth_foundational::facade::DurableStateFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryFamilySupport {
    Supported(u8),
    RefusedUntil(u8),
}

/// Base support does not admit later correction/component extensions.
pub const fn directory_family_support(family: DurableStateFamily) -> DirectoryFamilySupport {
    use DurableStateFamily::*;
    match family {
        ProgramActivation | PortablePackageMeaning => DirectoryFamilySupport::Supported(4),
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
        | AftermathCausality
        | CorrectionIntent
        | PublicationDiagnosticMaterial => DirectoryFamilySupport::Supported(4),
        // Phase 8's external dispatch custody is delivered for exactly an
        // emission's outbox row and its attempt journal.
        DispatchOutbox
        | DispatchAttempts
        | OutstandingDispatch
        | InboundSourceControls
        | InboundCompletionRows
        | InboundMessageCustody
        | InstalledTransportCompletionCustody
        | TerminalAcknowledgement => DirectoryFamilySupport::Supported(8),
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
        RecoveryClaims | RecoveryClaimDispositions | RecoveryHandleCustody | BankRecoveryClaim => {
            DirectoryFamilySupport::Supported(8)
        }
        CorrectionPreImages | CorrectionGovernedInput | CorrectionOriginalMeaning => {
            DirectoryFamilySupport::Supported(9)
        }
        OriginalOperationDescription
        | FixedDeliveryCustody
        | ServerSourceCustody
        | HttpOriginalOwners
        | UiStatusMutationOwner => DirectoryFamilySupport::RefusedUntil(9),
    }
}

/// Operation intent can require a later extension of an already-supported family.
///
/// Phase 8's external dispatch custody is delivered for exactly the outbox
/// row and the attempt journal. A non-inbound emission and dispatch are
/// admitted, and their recorded custody survives graceful close and reopen.
/// Inbound completion and recovery claims retain their own family requirements.
pub(crate) fn directory_requirement_refusal(
    family: DurableStateFamily,
    required_phase: u8,
) -> Option<u8> {
    match directory_family_support(family) {
        DirectoryFamilySupport::Supported(phase) => {
            if required_phase <= phase {
                None
            } else {
                Some(required_phase)
            }
        }
        DirectoryFamilySupport::RefusedUntil(phase) => Some(phase.max(required_phase)),
    }
}
