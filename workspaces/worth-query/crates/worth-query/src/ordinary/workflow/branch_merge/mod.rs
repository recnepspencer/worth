mod context;
mod declaration;
mod execution;
mod outcome;

pub use context::{branch_merge, WorthQueryBranchMergeContext, WorthQueryBranchMergeContextStop};
pub use declaration::{
    declare_branch_merge, WorthQueryBranchMergeDeclaration,
    WorthQueryBranchMergeDeclarationDenialKind, WorthQueryBranchMergeDeclarationIdentity,
    WorthQueryBranchMergeDeclarationStop,
};
pub use outcome::{
    WorthQueryBranchMergeAftermath, WorthQueryBranchMergeCompletion,
    WorthQueryBranchMergeControlStopped, WorthQueryBranchMergeDeferred,
    WorthQueryBranchMergeDurabilityUnresolved, WorthQueryBranchMergeNextAction,
    WorthQueryBranchMergeOutcome, WorthQueryBranchMergeStop, WorthQueryBranchMergeStopSource,
};

pub struct WorthQueryBranchMergeRequest {
    pub(crate) declaration: WorthQueryBranchMergeDeclaration,
    pub(crate) context: WorthQueryBranchMergeContext,
}
