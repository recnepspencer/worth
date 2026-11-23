//! Execution-time birth-plan, posture, and group transition assembly over
//! lowered installed name schemes.

mod assembly;
mod group;
mod postures;

pub use assembly::{
    ExecutionNameBirthPlan, ExecutionNameBirthPlanBuilder, ExecutionPendingEntityBirth,
    ExecutionPendingRelationBirth,
};
pub use group::{ExecutionGroupMemberSpec, ExecutionGroupTransitionOutcome};
pub use postures::{ExecutionNamedMergeKeeper, ExecutionSplitChildSpec};
