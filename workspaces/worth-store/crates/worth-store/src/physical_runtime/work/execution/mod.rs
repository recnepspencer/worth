mod command;
pub(in crate::physical_runtime) use command::PhysicalInspectionExecutorCommand;
mod joined_outcome;
mod outcome;
pub(super) mod settlement;

pub(in crate::physical_runtime) use command::{
    PhysicalCheckpointExecutorCommand, PhysicalMetadataExecutorCommand,
    PhysicalPublicationExecutorCommand, PhysicalReadExecutorCommand,
    PhysicalResidencyWritebackExecutorCommand, PhysicalRetryPayload,
    PhysicalWalAppendExecutorCommand, PhysicalWalBarrierExecutorCommand,
    PhysicalWalFrameCompletionBinding, PhysicalWalReclamationExecutorCommand,
    PhysicalWalSegmentCreateExecutorCommand, PhysicalWriteExecutorCommand,
};
pub use command::{
    PhysicalExecutorCommand, PhysicalExecutorCommandDenial, PhysicalPublicationEffect,
    PhysicalRetryCommand,
};
pub use joined_outcome::{
    PhysicalSignalSettlementOutcome, PhysicalWorkBatchDenial, PhysicalWorkExecutionBatchOutcome,
    PhysicalWorkExecutionOutcome,
};
pub use outcome::{
    CompletedPhysicalCheckpointAction, CompletedPhysicalPublicationEffect,
    CompletedPhysicalWalBarrier, CompletedPhysicalWalReclamationAction,
};
pub(in crate::physical_runtime) use outcome::{
    IndeterminatePhysicalCheckpointAction, IndeterminatePhysicalPublicationEffect,
    IndeterminatePhysicalWalBarrier, IndeterminatePhysicalWalReclamationAction,
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalResidencyWritebackCompletion,
};
pub(in crate::physical_runtime) use settlement::PhysicalWorkSettlement;
pub use settlement::{
    PhysicalWorkEffectFate, PhysicalWorkHealthRevocation, PhysicalWorkNoEffectEvidence,
    PhysicalWorkPublicationResiduePosture, PhysicalWorkSchedulerPosture,
    PhysicalWorkSettlementEvidence, PhysicalWorkTerminalCause, PhysicalWorkTerminalFailure,
};
