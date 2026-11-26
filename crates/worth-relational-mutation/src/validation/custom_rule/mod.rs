//! Runtime-bound custom invariant planning and execution.

pub mod affected_record_filter;
pub mod derived_changes;
pub mod execution_context;
pub use worth_relational_contracts::internal_port::validation::custom_rule::interruption::CustomInvariantInterruption;
pub mod read_budget;
pub use read_budget::{CustomInvariantNativeCheckpointDenial, CustomInvariantReadBudget};
pub mod managed_source;
pub mod persistent_names;
#[cfg(test)]
pub mod provenance_decode_tests;
pub mod registration;
pub mod scope_planner;
pub mod structural_source;
pub mod structural_views;
#[cfg(test)]
pub mod tests;
pub mod touched_scope_collection;
pub mod traversal;
#[cfg(test)]
pub mod work_budget_tests;
pub mod work_meter;

pub use execution_context::{
    CustomInvariantExecutionContext, CustomInvariantNonApplicability, CustomInvariantProvenance,
};
pub use persistent_names::{
    CustomInvariantNamePosture, CustomInvariantNameReadDenial, CustomInvariantNameReads,
    CustomInvariantNameSource, CustomInvariantNameView, CustomInvariantRecordName,
};
pub use registration::{
    CustomInvariantLeaseBudget, CustomInvariantRegistration, CustomInvariantRegistrationError,
    CustomInvariantRule,
};
pub use scope_planner::CustomInvariantScopePlanner;
pub(crate) use scope_planner::PreparedCustomInvariantScope;
pub use structural_views::{
    StructuralAspectStateView, StructuralReadError, StructuralRelationRecord,
    StructuralRelationView,
};
pub use traversal::{
    CompleteStructuralWalk, CustomInvariantTraversalSummary, StructuralWalkDenial,
    StructuralWalkDenialKind, StructuralWalkReads, StructuralWalkStep,
};

pub(crate) use registration::PreparedCustomInvariantExecution;
pub use work_meter::CustomInvariantWorkMeter;
pub use work_meter::CustomPreparationBudget;
pub(crate) use work_meter::CustomPreparationStop;
