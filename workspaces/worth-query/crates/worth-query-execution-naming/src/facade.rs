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
//! A native record handle is never a discriminator field, so no name can
//! depend on allocation:
//!
//! ```compile_fail,E0277
//! use worth_query_declaration::facade::application_naming::ApplicationNameDiscriminatorEncoder;
//! use worth_relational::facade::identity::EntityId;
//! fn smuggle(encoder: &mut ApplicationNameDiscriminatorEncoder, record: EntityId) {
//!     encoder.field("record", &record);
//! }
//! ```
//!
//! A birth under a typed role is born under the role's declared name; an
//! indexed role's index tells its repetitions apart:
//!
//! ```
//! use worth_query_declaration::facade::application_naming::{
//!     OccurrencePath, OccurrencePathDenial, WorthQueryNameRole,
//! };
//! use worth_query_execution_naming::facade::WorthQueryOutputBirth;
//!
//! #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
//! struct Instance(u32);
//! impl WorthQueryNameRole for Instance {
//!     const ROLE_ORDER: &'static [&'static str] = &["Instance"];
//!     const ROLE_DEPENDENCIES: &'static [(&'static str, &'static str)] = &[];
//!     fn role_name(&self) -> &'static str {
//!         "Instance"
//!     }
//!     fn role_index(&self) -> Option<u32> {
//!         Some(self.0)
//!     }
//!     fn from_role_parts(role: &str, index: Option<u32>) -> Option<Self> {
//!         (role == "Instance").then_some(Self(index?))
//!     }
//! }
//!
//! let occurrence = OccurrencePath::root("pattern_1")?.index("instance", 10)?;
//! let (second, tenth) = (
//!     WorthQueryOutputBirth::of_role(&occurrence, Instance(2)),
//!     WorthQueryOutputBirth::of_role(&occurrence, Instance(10)),
//! );
//! assert_eq!(tenth.occurrence_key(), "pattern_1/instance[10]");
//! assert_eq!((second.role_path(), tenth.role_path()), ("Instance", "Instance"));
//! assert!(second.has_discriminator() && second != tenth);
//! # Ok::<(), OccurrencePathDenial>(())
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_execution_naming::facade::AdmittedCommitNames;
//! use worth_relational::facade::runtime::RelationalRuntime;
//! use worth_relational::facade::transactions::CommitResult;
//! fn try_forge(runtime: &RelationalRuntime, result: &CommitResult) -> AdmittedCommitNames {
//!     AdmittedCommitNames::from_committed_result(runtime, result)
//! }
//! ```

pub use crate::admission::{same_commit_name_of, AdmittedCommitNames, ObservedEntityNames};
pub use crate::birth_plan::{
    ExecutionNameBirthPlan, ExecutionNameBirthPlanBuilder, ExecutionNamedMergeKeeper,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth, ExecutionSplitChildSpec,
};
pub use crate::commit_wiring::{CommittedNamedTransaction, ExecutionNamingCommitAuthority};
pub use crate::denial::ExecutionNamingDenial;
pub use crate::lowering::{LoweredExecutionGroupScheme, LoweredExecutionNameSchemeCatalog};
pub use crate::operation_outputs::{
    admit_output_birth, admit_same_commit_names, AuthoredOperationNaming, BoundOperationNaming,
    OperationMergeKeeper, OperationNamedOutput, OperationNamingEntry, OperationNamingProgram,
    OperationOutputKey, OutputBirthPosture, WorthQueryLeastMergeKeeper, WorthQueryOutputBirth,
    WorthQueryOutputMergeKeeper, WorthQueryOutputNameDenial,
};
pub use crate::tracked_resolution::{
    ExecutionNameReadDenial, ExecutionNameReader, ExecutionNameResolveDenial, NameReadCurrentness,
    NameReadDependencies, NameReadStaleness, NameResolutionCandidates,
};
/// Relational's commit-time refusals of an operation's declared output names,
/// as a commit denial carries them.
pub use worth_relational::facade::lineage::{
    DiscriminatorBasisDenialReason, LineageNameDenial, LineagePostureKind, LineageRecordTarget,
    PersistentNameHolder,
};
