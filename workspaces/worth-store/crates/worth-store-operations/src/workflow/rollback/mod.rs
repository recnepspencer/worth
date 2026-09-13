mod execution;
mod intent;
mod lowering;
mod owner_receipt;
mod recovery;

pub use execution::{
    ExecutedRollback, ExecutionReadyRollback, RollbackExecutionDenial, RollbackOperationReceipt,
    RollbackReadinessDenial,
};
pub use intent::{
    AdmittedRollbackSourceOperation, EvidenceBoundRollbackPlan, ResolvedRollbackOperation,
    RollbackIntent, RollbackResolutionDenial, RollbackSourceAdmissionDenial,
};
pub use lowering::{AuthorizedRollbackPlan, LoweredRollbackPlanDag, RollbackLoweringDenial};
pub(crate) use owner_receipt::rollback_owner_receipt_identity;
pub use recovery::{
    ResolvedRollbackCandidate, RollbackExecutionReceipt, RollbackReplayDenial, RollbackReplayOwner,
    RollbackReplayPlan,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RollbackOperation;
