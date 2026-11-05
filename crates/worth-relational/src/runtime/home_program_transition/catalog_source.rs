//! Borrowed protected intent DATA, never a runtime port or an execution grant.
use crate::branch::RecordedRelationalComponentReference;
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
};

#[derive(Debug)]
pub struct RelationalHomeProgramCatalogReadmissionSource<'intent> {
    plan: &'intent HomeProgramTransitionPlanDescription,
    predecessor: &'intent RecordedRelationalComponentReference,
    assignments: &'intent [CompositeAttemptAssociation],
    callers: worth_execution::ExecutionArray<[u8; 32]>,
}
impl<'intent> RelationalHomeProgramCatalogReadmissionSource<'intent> {
    /// Describing data grants nothing. Only the actual Home can seal its source.
    pub fn from_recorded(
        plan: &'intent HomeProgramTransitionPlanDescription,
        predecessor: &'intent RecordedRelationalComponentReference,
        assignments: &'intent [CompositeAttemptAssociation],
        callers: worth_execution::ExecutionArray<[u8; 32]>,
    ) -> Self {
        Self {
            plan,
            predecessor,
            assignments,
            callers,
        }
    }
    pub fn plan(&self) -> &HomeProgramTransitionPlanDescription {
        self.plan
    }
    pub fn predecessor(&self) -> &RecordedRelationalComponentReference {
        self.predecessor
    }
    pub fn callers(&self) -> &[[u8; 32]] {
        self.callers.elements()
    }
    pub fn assignments(&self) -> &[CompositeAttemptAssociation] {
        self.assignments
    }
}
