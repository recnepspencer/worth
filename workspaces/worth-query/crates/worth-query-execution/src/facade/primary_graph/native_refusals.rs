//! Native refusal values retained by the Query audience.
pub use worth_relational::facade::branch::RelationalModelOwnershipDenial;
/// Original Native checkpoint recovery classes, retained by installation denials.
pub use worth_relational::facade::durability::{
    DurabilityError, RecoveryFailureClass, RelationalRecoveryStop,
};
pub use worth_relational::facade::errors::{ErrorContext, ErrorOperation, RelationalSubsystem};
pub use worth_relational::facade::mvcc::{
    RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin,
};
pub use worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial;
pub use worth_relational::facade::runtime::{
    CustomInvariantAccessDenial, CustomInvariantAccessDenialKind, CustomInvariantFailureCause,
};
pub use worth_relational::facade::transactions::{
    CommitExecutionDenial, CommitExecutionDenialKind, CommitPreparationReason,
    CustomInvariantFailurePhase, ResultCustomInvariantFailureKind,
};
