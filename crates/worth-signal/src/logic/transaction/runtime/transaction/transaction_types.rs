mod evidence;
mod handles;
mod outcome;
mod rollback;
mod state;

pub use evidence::{TemporalEligibilityFact, TemporalTransactionEvidence};
pub(in crate::logic::transaction::runtime) use handles::TransactionCommitPosture;
pub use handles::{BatchChangeSession, SignalTransaction};
pub use outcome::{
    EvaluationSummary, TransactionOutcome, TransactionReplayEntry, TransactionResult,
    TransactionTiming,
};
pub(in crate::logic::transaction::runtime) use rollback::{
    CreatedNodeRollbackDelta, GraphPatchRollbackDelta, SubscriberRepairRollbackDelta,
    TransactionRollbackPacket, TransactionRollbackPacketSet,
};
pub(in crate::logic::transaction::runtime) use state::{
    StagedEventOperation, TransactionExecutionState, TransactionScratch,
};
#[cfg(test)]
#[path = "transaction_types_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
