//! Installed operation intents distinguish extension support from base state evidence.
use worth_foundational::facade::DurableStateFamily as Family;

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
impl WorthQueryOperationDurableCapability {
    pub(in crate::domain_computation) const fn requirements(self) -> (&'static [Family], u8) {
        match self {
            Self::RelationalMutation => (
                &[
                    Family::QueryIdempotencyBinding,
                    Family::WorldAttemptProgress,
                    Family::WorldUnpublishedCustody,
                ],
                4,
            ),
            Self::SignalMutation => (&[Family::SignalBranchCells], 5),
            Self::BridgeMutation => (
                &[
                    Family::BridgeSemanticRegistrations,
                    Family::BridgeCorrespondenceAllocation,
                ],
                5,
            ),
            Self::RequiredOutputDemand => (
                &[Family::OutputDemandRecords, Family::OutputSourceCustody],
                6,
            ),
            Self::ProgramAdoption => (
                &[
                    Family::ProgramSupportLifecycle,
                    Family::BranchAdoptionRecovery,
                ],
                6,
            ),
            Self::ManagedExecution => (
                &[Family::ManagedDirectRuns, Family::PendingApplicationCleanup],
                7,
            ),
            Self::ResourceRequest => (
                &[
                    Family::ResourceOccurrences,
                    Family::ResourceIngress,
                    Family::SignalResourceRequestCustody,
                ],
                7,
            ),
            Self::ConditionalExecution => (
                &[
                    Family::ConditionalDecisions,
                    Family::RetainedConditionalCleanup,
                ],
                7,
            ),
            Self::TemporalExecution => (
                &[Family::ConditionalDecisions, Family::TemporalIdempotency],
                7,
            ),
            Self::WorkflowExecution => {
                (&[Family::WorkflowInstances, Family::WorkflowTransitions], 7)
            }
            Self::ExternalEmission => (&[Family::DispatchOutbox], 8),
            Self::ExternalDispatch => (&[Family::DispatchAttempts, Family::OutstandingDispatch], 8),
            Self::InboundCompletion => (
                &[
                    Family::OutstandingDispatch,
                    Family::InboundCompletionRows,
                    Family::InboundMessageCustody,
                    Family::InstalledTransportCompletionCustody,
                    Family::TerminalAcknowledgement,
                    Family::InboundSourceControls,
                ],
                8,
            ),
            Self::RecoveryCustody => (
                &[
                    Family::RecoveryHandleCustody,
                    Family::RecoveryClaims,
                    Family::RecoveryClaimDispositions,
                ],
                8,
            ),
            Self::RetainedCorrectionMaterial => (
                &[
                    Family::CorrectionGovernedInput,
                    Family::CorrectionPreImages,
                    Family::CorrectionOriginalMeaning,
                ],
                9,
            ),
            Self::RetainedOriginalOperationOwner => (
                &[
                    Family::OriginalOperationDescription,
                    Family::SourceOperationCustody,
                ],
                9,
            ),
            Self::NativeBranchCleanup => (&[Family::ApplicationBranchCleanup], 4),
            Self::SignalBranchCleanup => (&[Family::ApplicationBranchCleanup], 5),
            Self::ConditionalBranchCleanup => (&[Family::ApplicationBranchCleanup], 7),
        }
    }
}
