//! Narrow public facade for execution-time persistent naming lowering,
//! birth-plan assembly, Relational commit wiring, and kind-verified admitted
//! name extraction from sealed commits.
//!
//! Callers cannot construct `AdmittedCommitNames` or forge a `WorthQueryName<K>`
//! from raw bytes:
//!
//! ```compile_fail
//! use worth_query_declaration::facade::application_naming::WorthQueryName;
//! struct Face;
//! let _forged: WorthQueryName<Face> = WorthQueryName::from_bytes([7u8; 32]);
//! ```
//!
//! ```compile_fail
//! use worth_query_execution_naming::facade::AdmittedCommitNames;
//! let _forged = AdmittedCommitNames::from_committed_result();
//! ```

pub use crate::admission::AdmittedCommitNames;
pub use crate::birth_plan::{
    ExecutionNameBirthPlan, ExecutionNameBirthPlanBuilder, ExecutionNamedMergeKeeper,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth, ExecutionSplitChildSpec,
};
pub use crate::commit_wiring::{CommittedNamedTransaction, ExecutionNamingCommitAuthority};
pub use crate::denial::ExecutionNamingDenial;
pub use crate::lowering::{LoweredExecutionGroupScheme, LoweredExecutionNameSchemeCatalog};
