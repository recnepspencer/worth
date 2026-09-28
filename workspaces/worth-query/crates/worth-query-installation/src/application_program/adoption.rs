//! What moving a branch from one rostered program to another demands of the
//! state that branch already holds.
//!
//! Adoption is a change of governing meaning, not a change of data. The only
//! thing it can break is state that was legal under the program the branch was
//! running and is not legal under the one it is moving to, so what installation
//! owes the operation is the exact set of rules that start governing and the
//! scope each of them reaches. Everything else — deciding whether the branch is
//! allowed to move, reading the state, publishing the result — belongs to
//! execution and World, not here.

mod requirements;
mod scope_validation;
mod workflow_dependencies;

#[cfg(test)]
mod requirements_tests;

pub use requirements::{
    WorthQueryProgramAddedRule, WorthQueryProgramAdoptionRequirements,
    WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramCustodyInventoryKind,
    WorthQueryProgramCustodyInventoryRequirement, WorthQueryProgramValidationScope,
};
pub use workflow_dependencies::WorthQueryWorkflowDependencyName;
