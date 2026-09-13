mod admission;
mod aspect_delta;
mod authority;
mod command_storage;
mod concurrency_scope;
mod consumer_lifecycle;
mod declaration;
mod drain_observation;
mod execution;
pub(in crate::physical_runtime) use execution::PhysicalInspectionExecutorCommand;
mod identity;
mod observation;
mod profile;
mod progression;
mod recovery;
mod request;
mod scheduler_demand;
mod semantic_basis;
mod signal_declaration;
mod submission;
pub(in crate::physical_runtime) use submission::{
    physical_work_abandonment_channel, PhysicalEffectActivity, PhysicalWorkAbandonmentInbox,
    PhysicalWorkAbandonmentPublisher, PhysicalWorkAbandonmentWake, PhysicalWorkSafeCancellation,
};

pub use authority::AdmittedPhysicalWorkAuthority;
pub(in crate::physical_runtime) use authority::PhysicalWorkAdmissionAuthority;
pub use concurrency_scope::{PhysicalWorkConcurrencyRelation, PhysicalWorkConcurrencyScope};
pub use consumer_lifecycle::{
    PhysicalEffectObligation, PhysicalWorkCancellationFailure, PhysicalWorkCancellationJoin,
    PhysicalWorkConsumerHandle, PhysicalWorkRetryAdmission, PhysicalWorkRetryFailure,
    PhysicalWorkRetrySchedule, PhysicalWorkRetryScheduleOutcome, PhysicalWorkSupersessionJoin,
    PhysicalWorkTimeoutJoin,
};
pub(in crate::physical_runtime) use declaration::{
    PhysicalCheckpointWorkAction, PhysicalCheckpointWorkScope, PhysicalRootPublicationWorkAction,
    PhysicalRootPublicationWorkScope, PhysicalWalReclamationScope,
};
pub use declaration::{
    PhysicalWalAppendScope, PhysicalWalBarrierScope, PhysicalWalFrameWriteDisposition,
    PhysicalWorkDeclarationDenial, PhysicalWorkDurabilityRequirement, PhysicalWorkEffectClass,
    PhysicalWorkIntent, PhysicalWorkOperationFamily, PhysicalWorkRecoveryDisposition,
    PhysicalWorkScope,
};
pub use drain_observation::PhysicalWorkDrainObservation;
use drain_observation::{PhysicalWorkTerminalEvent, PhysicalWorkTerminalLedger};
pub use execution::{
    CompletedPhysicalCheckpointAction, CompletedPhysicalPublicationEffect,
    CompletedPhysicalWalBarrier, CompletedPhysicalWalReclamationAction, PhysicalExecutorCommand,
    PhysicalExecutorCommandDenial, PhysicalPublicationEffect, PhysicalRetryCommand,
    PhysicalSignalSettlementOutcome, PhysicalWorkBatchDenial, PhysicalWorkEffectFate,
    PhysicalWorkExecutionBatchOutcome, PhysicalWorkExecutionOutcome, PhysicalWorkHealthRevocation,
    PhysicalWorkNoEffectEvidence, PhysicalWorkPublicationResiduePosture,
    PhysicalWorkSchedulerPosture, PhysicalWorkSettlementEvidence, PhysicalWorkTerminalCause,
    PhysicalWorkTerminalFailure,
};
pub use identity::{
    PhysicalEffectIdentity, PhysicalOperationIdentity, PhysicalWorkGeneration, PhysicalWorkIdentity,
};
pub use observation::{
    PhysicalWorkCausalObservation, PhysicalWorkCausalRecord, PhysicalWorkCounterSnapshot,
    PhysicalWorkCounterStage, PhysicalWorkObservation, PhysicalWorkPressureClass,
    PhysicalWorkShutdownObservation, PhysicalWorkTerminalDisposition,
    PhysicalWorkTerminalObservation, PhysicalWorkTerminalStage,
};
pub use profile::{
    PhysicalSignalAspectBinding, PhysicalSignalAspectBindingDigest,
    PhysicalSignalAspectBindingObservation, PhysicalSignalAspectDeclaration,
    PhysicalSignalAspectRole, PhysicalSignalAspectSubscription, PhysicalSignalBindingDenial,
    PhysicalSignalProfileIdentity, PhysicalWorkCapacity, PhysicalWorkProfileDeclaration,
    PhysicalWorkProfileDenial, PhysicalWorkSignalFamily, PhysicalWorkSignalFamilySet,
};
pub(in crate::physical_runtime) use progression::PhysicalSignalReadinessEvidence;
pub use progression::{
    AdmittedPhysicalWork, BlockedPhysicalWork, DispatchedPhysicalWork, PhysicalWorkReadiness,
    ReadyPhysicalWork, ResourceAdmittedPhysicalWork, SettledPhysicalWork,
};
pub use recovery::PhysicalCheckpointRecoveryAction;
pub(in crate::physical_runtime) use recovery::{
    PhysicalEffectJournal, PhysicalEffectRecoveryInventory, PreparedPhysicalEffect,
};
pub use recovery::{
    PhysicalWorkRecoveryAdmissionCounters, PhysicalWorkRecoveryAdmissionObservation,
    PhysicalWorkRecoveryAdmissionOutcome, PhysicalWorkRecoveryIngressRejection,
    PhysicalWorkRecoveryObservationSubject,
};
pub use recovery::{PhysicalWorkRecoveryLocator, PhysicalWorkRecoveryTarget};
pub use request::{
    PhysicalMetadataReadWorkRequest, PhysicalMutationWorkRequest, PhysicalReadWorkRequest,
};
pub use scheduler_demand::{
    PhysicalSchedulerDemand, PhysicalSchedulerDenial, PhysicalWorkScheduler,
};
pub use semantic_basis::{
    PhysicalWorkSemanticBasis, PhysicalWorkSemanticBasisDenial, PhysicalWorkSemanticPosture,
};
pub use signal_declaration::PhysicalWorkSignalDeclaration;
pub use submission::{
    PhysicalMutationSubmission, PhysicalReadSubmission, PhysicalWorkCapacityDimension,
    PhysicalWorkSubmissionDeferred, PhysicalWorkSubmissionDenial, PhysicalWorkSubmissionFailure,
    PhysicalWorkSubmissionOutcome, PhysicalWorkSubmissionReceipt, PhysicalWorkSubmissionStale,
};

pub use admission::{PhysicalWorkAdmission, PhysicalWorkPreEffectDenial};
pub use aspect_delta::{PhysicalWorkAspectDelta, PhysicalWorkAspectDeltaDenial};
pub(in crate::physical_runtime) use declaration::PhysicalWorkIntentParts;
pub(in crate::physical_runtime) use execution::{
    IndeterminatePhysicalCheckpointAction, IndeterminatePhysicalPublicationEffect,
    IndeterminatePhysicalWalBarrier, IndeterminatePhysicalWalReclamationAction,
    PhysicalCheckpointExecutorCommand, PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch,
    PhysicalExecutorOutcome, PhysicalMetadataExecutorCommand, PhysicalPublicationExecutorCommand,
    PhysicalReadExecutorCommand, PhysicalResidencyWritebackCompletion,
    PhysicalResidencyWritebackExecutorCommand, PhysicalRetryPayload,
    PhysicalWalAppendExecutorCommand, PhysicalWalBarrierExecutorCommand,
    PhysicalWalFrameCompletionBinding, PhysicalWalReclamationExecutorCommand,
    PhysicalWalSegmentCreateExecutorCommand, PhysicalWorkSettlement, PhysicalWriteExecutorCommand,
};
pub use profile::PhysicalSignalAspectBindingSet;
pub(in crate::physical_runtime) use profile::PHYSICAL_ASYNC_CAPABILITIES;
pub(in crate::physical_runtime) use signal_declaration::{
    InstalledPhysicalSignalTopology, PendingPhysicalSignalTopology,
};
#[cfg(feature = "certification-test-authority")]
pub use submission::CertificationPhysicalSubmissionPauseGate;
pub(in crate::physical_runtime) use submission::{
    PhysicalMutationIdentityReservationError, PhysicalWorkStopKind,
    PhysicalWorkSubmissionFoundation, PhysicalWorkSubmissionOwner,
};
