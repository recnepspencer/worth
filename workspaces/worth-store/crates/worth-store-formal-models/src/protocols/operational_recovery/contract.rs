#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalRecoveryInvariant {
    UniqueTransitionIdentity,
    SingleWorkflowOpen,
    SourceLeaseBeforeMaterialization,
    MaterializationOpenBeforeReceipt,
    MaterializationBeforeVerification,
    AuthorizationBeforeExecution,
    OwnerExecutionBeforeEffect,
    OwnerExecutionBeforeReceipt,
    OwnerEffectBeforeReceipt,
    WorkflowBeforeOwnerReceipt,
    CompleteOwnerReceiptsBeforeStaging,
    TerminalOperationHasNoLaterTransition,
    PreparationBeforePublication,
    PendingBeforeDisposition,
    DispositionBeforeFenceRelease,
    ExternalFenceBeforePromotion,
    BootstrapTransferBeforeCompletion,
    PromotionBeforePublication,
    PromotionPublicationBeforeReadmission,
    PromotionReadmissionBeforeRejoin,
    RejoinPlanBeforeCompletion,
    AuthorizationReplayRejected,
    AuthorizationPlanBindingPreserved,
    PublicationBindingPreserved,
    BootstrapBindingPreserved,
    PromotionBindingPreserved,
    PromotionEpochMonotonic,
    RejoinBindingPreserved,
    RejoinDispositionComplete,
    SemanticIdentityNonZero,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalRecoveryCounterexample {
    pub(super) operation_identity: String,
    pub(super) transition_identity: String,
    pub(super) invariant: OperationalRecoveryInvariant,
}

impl OperationalRecoveryCounterexample {
    pub fn operation_identity(&self) -> &str {
        &self.operation_identity
    }
    pub fn transition_identity(&self) -> &str {
        &self.transition_identity
    }
    pub const fn invariant(&self) -> OperationalRecoveryInvariant {
        self.invariant
    }
}
