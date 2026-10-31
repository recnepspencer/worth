//! Semantic durable admission families from the M3 survival audit.
//! These names describe state obligations; they grant no runtime authority.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DurableStateFamily {
    /// Audited base/extension target: 4 graph authority; 9 original-owner advancement.
    AggregateIdentity,
    /// Audited base/extension target: 4 graph authority; 9 original-owner advancement.
    AggregateMembership,
    /// Audited base/extension target: 4 graph authority; 9 correction integration.
    AggregateLineage,
    /// Audited base/extension target: 4 graph authority; 9 original-owner advancement.
    AggregateProjectedCoordinates,
    /// Audited base/extension target: 4.
    RelationalPendingSettlement,
    /// Audited base/extension target: 4.
    WorldIdentityFrontiers,
    /// Audited base/extension target: 4.
    WorldProductBranches,
    /// Audited base/extension target: 4.
    WorldProductHeads,
    /// Audited base/extension target: 4.
    WorldCompositeHistory,
    /// Audited base/extension target: 4.
    WorldPerformedPublication,
    /// Audited base/extension target: 4; Signal component 5.
    WorldAttemptProgress,
    /// Audited base/extension target: 4; mixed/conditional definitions 5.
    WorldUnpublishedCustody,
    /// Audited base/extension target: 4; Signal forks 5.
    WorldComponentBranchCustody,
    /// Audited base/extension target: 5.
    WorldConditionalDefinitionCustody,
    /// Audited base/extension target: 5.
    SignalNodeDefinitions,
    /// Audited base/extension target: 5, authored definitions + declared static edges only (R2).
    SignalTopology,
    /// Audited base/extension target: 5.
    SignalBranchCells,
    /// Audited base/extension target: 5.
    SignalOwnerFrontiers,
    /// Audited base/extension target: 5.
    SignalBranchMetadata,
    /// Audited base/extension target: 7.
    SignalOwnedAsyncCustody,
    /// Audited base/extension target: 7; actual queue/resource is freshly admitted.
    SignalOwnedAsyncQueue,
    /// Audited base/extension target: 5.
    SignalRuntimeDefinitions,
    /// Audited base/extension target: 5.
    BridgeConditionalDefinitions,
    /// Audited base/extension target: 5.
    BridgeSemanticRegistrations,
    /// Audited base/extension target: 5.
    BridgeCorrespondenceAllocation,
    /// Audited base/extension target: 7.
    BridgeManagedTemporalIntents,
    /// Audited base/extension target: 5 definitions; operations 7.
    BridgeOwnedAsyncDefinitions,
    /// Audited base/extension target: 7.
    BridgeOwnedAsyncCustody,
    /// Audited base/extension target: 5.
    BridgeDiagnosticRetention,
    /// Audited base/extension target: 4; temporal/source specializations 7/6.
    QueryIdempotencyBinding,
    /// Audited base/extension target: 4; mixed owner extension 5.
    UnpublishedIdempotency,
    /// Audited base/extension target: 4; correction entries 9.
    PendingApplicationPublication,
    /// Audited base/extension target: 3 binding; 6 every-branch verification.
    ProgramActivation,
    /// Audited base/extension target: 6, query.obligations.
    ProgramSupportLifecycle,
    /// Audited base/extension target: 6.
    BranchAdoptionRecovery,
    /// Audited base/extension target: 6.
    StoppedBranchSetAdoption,
    /// Audited base/extension target: 4 World native / 5 Signal / 7 conditional assembly (R12).
    ApplicationBranchCleanup,
    /// Audited base/extension target: 7, query.obligations.
    PendingApplicationCleanup,
    /// Audited base/extension target: 7 (underlying unpublished association 4/5).
    ProducerCleanupCustody,
    /// Audited base/extension target: 7.
    ConditionalDecisions,
    /// Audited base/extension target: 7.
    RetainedConditionalCleanup,
    /// Audited base/extension target: 7.
    TemporalIdempotency,
    /// Audited base/extension target: 7.
    ResourceOccurrences,
    /// Audited base/extension target: 7.
    ResourceIngress,
    /// Audited base/extension target: 7.
    OwnedAsyncCustody,
    /// Audited base/extension target: 7.
    ManagedDirectRuns,
    /// Audited base/extension target: 7.
    ManagedWorkflowRuns,
    /// Audited base/extension target: 7.
    YieldedDirectRuns,
    /// Audited base/extension target: 7.
    YieldedWorkflowRuns,
    /// Audited base/extension target: 7.
    ProviderSuspensions,
    /// Audited base/extension target: 7.
    DirectYieldRecovery,
    /// Audited base/extension target: 7.
    WorkflowYieldRecovery,
    /// Audited base/extension target: 7.
    ManagedRestoreRecovery,
    /// Audited base/extension target: 7.
    DirectTerminalCleanup,
    /// Audited base/extension target: 7.
    WorkflowTerminalCleanup,
    /// Audited base/extension target: 4 receipt evidence; 8 dispatch semantics.
    DispatchOutbox,
    /// Audited base/extension target: 8, query.dispatch before external contact.
    DispatchAttempts,
    /// Audited base/extension target: 8.
    OutstandingDispatch,
    /// Audited base/extension target: 8.
    InboundCompletionRows,
    /// Audited base/extension target: 8.
    InboundMessageCustody,
    /// Audited base/extension target: 8.
    InstalledTransportCompletionCustody,
    /// Audited base/extension target: 8.
    TerminalAcknowledgement,
    /// Audited base/extension target: 8, durable home security facts (R13).
    InboundSourceControls,
    /// Audited base/extension target: 8, query.dispatch.
    RecoveryClaims,
    /// Audited base/extension target: 8, durable claim/disposition records associated with query.dispatch.
    RecoveryClaimDispositions,
    /// Audited base/extension target: 8; retained correction payloads 9.
    RecoveryHandleCustody,
    /// Audited base/extension target: 4 identity foundation; 9 correction lineage readmission.
    AftermathCausality,
    /// Audited base/extension target: 4 for binding/World identity; 9 correction operation.
    CorrectionIntent,
    /// Audited base/extension target: 9, optional unpinned / mandatory while pinned (R11).
    CorrectionPreImages,
    /// Audited base/extension target: 9, optional unpinned / mandatory while pinned (R7/R11).
    CorrectionGovernedInput,
    /// Audited base/extension target: 9, optional unpinned / mandatory while pinned (R11).
    CorrectionOriginalMeaning,
    /// Audited base/extension target: 6 general required/prerequisite/performed obligations (R8); 7 managed; 9 correction.
    OutputDemandRecords,
    /// Audited base/extension target: 6 general source custody (R8); 9 correction-specific.
    OutputSourceCustody,
    /// Audited base/extension target: 7.
    WorkflowLineages,
    /// Audited base/extension target: 7.
    WorkflowDefinitions,
    /// Audited base/extension target: 7.
    WorkflowNodes,
    /// Audited base/extension target: 7.
    WorkflowConnections,
    /// Audited base/extension target: 7.
    WorkflowInstances,
    /// Audited base/extension target: 7.
    WorkflowTransitions,
    /// Audited base/extension target: 7.
    WorkflowProposals,
    /// Audited base/extension target: 7.
    WorkflowProposalCoverage,
    /// Audited base/extension target: 7.
    WorkflowAssessmentEvidence,
    /// Audited base/extension target: 7.
    WorkflowEvidenceDependencies,
    /// Audited base/extension target: 7.
    WorkflowApprovals,
    /// Audited base/extension target: 7.
    WorkflowOperationOutputs,
    /// Audited base/extension target: 7.
    WorkflowCapturedInputs,
    /// Audited base/extension target: 3/6 program binding and transition; supplied package readmission.
    PortablePackageMeaning,
    /// Audited base/extension target: 4/9 when retained with receipt/correction; policy must name expiry.
    PublicationDiagnosticMaterial,
    /// Audited base/extension target: 9, using 7 custody; fresh authenticated advancement.
    OriginalOperationDescription,
    /// Audited base/extension target: 7 for cleanup; 9 for same original owner.
    SourceOperationCustody,
    /// Audited base/extension target: 7; correction entries additionally 9.
    OperationCleanupCustody,
    /// Audited base/extension target: 6 general performed-source obligation (R8); 7 managed; 9 correction/original owner.
    PerformedSourcePreparation,
    /// Audited base/extension target: 9; reissue fresh owner for original identity.
    FixedDeliveryCustody,
    /// Audited base/extension target: 6 required-output obligations (R8); 7 mandatory producer cleanup; read lease stays local.
    ProgramOutputReadCustody,
    /// Audited base/extension target: 9, with managed foundation in 7.
    ServerSourceCustody,
    /// Audited base/extension target: 8; recovered claim then fresh handle.
    BankRecoveryClaim,
    /// Audited base/extension target: 7; fresh resource authority.
    BankManagedCleanup,
    /// Audited base/extension target: 9, using 7/8 for cleanup/external custody.
    HttpOriginalOwners,
    /// Audited base/extension target: 7; native obligation readmission, fresh permit.
    HttpNativeCleanup,
    /// Audited base/extension target: 7 logical Query custody; physical presentation is local (R16).
    UiPresentationCustody,
    /// Audited base/extension target: 7/9 when retained by an original operation; fresh provider admission otherwise.
    CapturedExternalInput,
    /// Audited base/extension target: 9; cleanup foundation in 7, fresh authenticated commands.
    UiStatusMutationOwner,
    /// Audited base/extension target: 7 cleanup/managed foundation; 9 same original owner.
    FixedOperationCustody,
    /// Audited base/extension target: 7 (R5); extracted before treating remaining resource state as derived.
    SignalResourceRequestCustody,
}
