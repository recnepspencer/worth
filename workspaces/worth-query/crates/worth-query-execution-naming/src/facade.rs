//! Narrow public facade for execution-time persistent naming lowering,
//! birth-plan assembly, Relational commit wiring, and admitted name elevation.

pub use crate::admission::{
    admit_group_assignment, admit_minted_birth_record, admit_resolved_name_key,
    AdmittedCommitNames,
};
pub use crate::birth_plan::{
    ExecutionGroupMemberSpec, ExecutionGroupTransitionOutcome, ExecutionNameBirthPlan,
    ExecutionNameBirthPlanBuilder, ExecutionNamedMergeKeeper, ExecutionPendingEntityBirth,
    ExecutionPendingRelationBirth, ExecutionSplitChildSpec,
};
pub use crate::commit_wiring::{CommittedNamedTransaction, ExecutionNamingCommitAuthority};
pub use crate::denial::ExecutionNamingDenial;
pub use crate::lowering::{LoweredExecutionGroupScheme, LoweredExecutionNameSchemeCatalog};
