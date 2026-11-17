//! Installed operation intents distinguish extension support from base state evidence.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationDurableCapability {
    RelationalMutation,
    SignalMutation,
    BridgeMutation,
    RequiredOutputDemand,
    ProgramAdoption,
    ManagedExecution,
    ResourceRequest,
    ConditionalExecution,
    TemporalExecution,
    WorkflowExecution,
    ExternalEmission,
    ExternalDispatch,
    InboundCompletion,
    RecoveryCustody,
    RetainedCorrectionMaterial,
    RetainedOriginalOperationOwner,
    NativeBranchCleanup,
    SignalBranchCleanup,
    ConditionalBranchCleanup,
}
