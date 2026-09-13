mod execution;
mod intent;
mod lowering;
mod owner_receipt;
mod recovery;

pub use execution::{
    BackupRestoreExecutionDenial, BackupRestoreReadinessDenial, ExecutedBackupRestore,
    ExecutionReadyBackupRestore, RestoreExecutionReceipt,
};
pub use intent::{BackupRestoreIntent, EvidenceBoundBackupRestorePlan};
pub use lowering::{
    AuthorizedBackupRestorePlan, BackupRestoreLoweringDenial, LoweredBackupRestorePlan,
};
pub(crate) use owner_receipt::restored_frontier_owner_receipt_identity;
pub use recovery::{
    BackupRestoreReplayDenial, BackupRestoreReplayOwner, BackupRestoreReplayPlan,
    BackupRestoreReplayRequest, RecoveredBackupFrontierReceipt,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct BackupRestoreOperation;
