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
        | BridgeDiagnosticRetention => DirectoryFamilySupport::Supported(5),

        ProgramSupportLifecycle => DirectoryFamilySupport::Supported(6),
        BranchAdoptionRecovery
        | StoppedBranchSetAdoption
        | OutputDemandRecords
        | OutputSourceCustody
        | PerformedSourcePreparation
        | ProgramOutputReadCustody => DirectoryFamilySupport::RefusedUntil(6),
        // The logical cleanup stays in Query's one journal. Native resources
        // must have an actual live release before graceful Home handoff.
        PendingApplicationCleanup
        | ConditionalDecisions
        | RetainedConditionalCleanup
        | TemporalIdempotency
        | BridgeManagedTemporalIntents
        | ManagedDirectRuns
        | DirectTerminalCleanup => DirectoryFamilySupport::Supported(7),
        // no Home installation admits generic managed providers; owner-boundary court yielded_owner_data_preserves_terminal_cause_and_refuses_unrecorded_release
        YieldedDirectRuns | ProviderSuspensions | DirectYieldRecovery => {
            DirectoryFamilySupport::Supported(7)
        }
        // no Home installation admits generic managed providers; owner-boundary court managed_restore_owner_data_keeps_original_failure_and_separate_release_facts
        ManagedRestoreRecovery => DirectoryFamilySupport::Supported(7),
        // no Home installation admits generic managed providers; owner-boundary court workflow_owner_data_retains_pending_artifacts_after_native_resource_release
        ManagedWorkflowRuns
        | YieldedWorkflowRuns
        | WorkflowYieldRecovery
        | WorkflowTerminalCleanup => DirectoryFamilySupport::Supported(7),
        BridgeOwnedAsyncDefinitions
        | SignalOwnedAsyncCustody
        | SignalOwnedAsyncQueue
        | BridgeOwnedAsyncCustody
        | ResourceOccurrences
        | ResourceIngress
        | OwnedAsyncCustody
        | SignalResourceRequestCustody => DirectoryFamilySupport::Supported(7),
        // No Home installation admits caller-owned generic external captures;
        // owner-boundary court external_input_capture_refusal_preserves_selection_values_revision_and_provenance.
        // Encoded inputs admitted to actual work use that work's durable record.
        CapturedExternalInput => DirectoryFamilySupport::Supported(7),
        // Query's logical request/queue custody is the shared owned-async owner;
        // physical UI presentation has no Home installation and is out of scope.
        // Owner-boundary court mandatory_owned_async_keeps_payload_generation_lineage_and_requires_fresh_owner.
        UiPresentationCustody => DirectoryFamilySupport::Supported(7),
        // These wrappers own the same Query cleanup obligation, not another
        // journal or resource codec. Their Native release/cold logical disposal
        // is qualified by failed_managed_cleanup_records_live_release_and_discharges_after_directory_reopen.
        ProducerCleanupCustody
        | OperationCleanupCustody
        | SourceOperationCustody
        | FixedOperationCustody
        | BankManagedCleanup
        | HttpNativeCleanup => DirectoryFamilySupport::Supported(7),
        WorkflowLineages
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
        | WorkflowCapturedInputs => DirectoryFamilySupport::RefusedUntil(7),
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
