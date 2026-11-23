//! Execution-time birth-plan and posture assembly over lowered installed name
//! schemes.

mod assembly;
mod postures;

pub use assembly::{
    ExecutionNameBirthPlan, ExecutionNameBirthPlanBuilder, ExecutionPendingEntityBirth,
    ExecutionPendingRelationBirth,
};
pub use postures::{ExecutionNamedMergeKeeper, ExecutionSplitChildSpec};
