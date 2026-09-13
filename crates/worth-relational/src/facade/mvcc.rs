//! Branch-qualified transaction preparation and publication surface.

pub use crate::mvcc::{
    BranchBoundRelationalTransaction, CustomInvariantExecutionReceipt,
    DiscardedRelationalCommitCandidate, PerformedRelationalCommit,
    PreparedRelationalCommitCandidate, PublishRelationalCommit, RelationalBranchObservation,
    RelationalBranchTransactionAdmissionDenial, RelationalCancellationSource,
    RelationalCancellationToken, RelationalInterruptionBoundary,
    RelationalInterruptionCostCounters, RelationalInterruptionEvent,
    RelationalMutationInvariantEvidence, RelationalOperationControl,
    RelationalOperationInterruption, RelationalPreparationPort, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationDurabilityPosture,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
    RelationalPublicationPort, RelationalPublicationProjectionPosture,
    RelationalTransactionEntityRead, RelationalTransactionFootprint, RelationalTransactionIntent,
    RelationalTransactionReadLocus, RelationalTransactionRelationRead,
    RelationalTransactionRelationValue, RelationalTransactionStagingDenial,
    RelationalTransactionWriteLocus, StaleRelationalBranchObservation, ValidatedMutationFootprint,
    ValidatedMutationFootprintNotRequested, ValidatedMutationFootprintProjection,
    ValidatedMutationFootprintWork, ValidatedMutationTouch, ValidatedMutationTouchProjectionError,
    ValidatedMutationTouchProjectionWork, ValidatedMutationTouches, ValidatedRelationalProposal,
};
pub use crate::publication::RelationalSettlementPort;
pub use crate::transactions::data::{
    CommitConflict, CommitResult, ConflictClass, TransactionCommitError, WorkerIntentBatch,
};

#[cfg(feature = "test-operation-control")]
pub use crate::runtime::RelationalPatchPositionReservationGate;
