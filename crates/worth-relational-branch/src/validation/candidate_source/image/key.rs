//! Canonical declared storage identity; live binding checks remain separate.
use super::{EntityId, Program};
use std::cmp::Ordering;
use worth_relational_contracts::internal_port::validation::candidate_source::program::ManagedIndexSourceProgramFieldsPort as _;
#[derive(Clone)]
pub struct SourceKey {
    program: Program,
    scope: EntityId,
}
impl SourceKey {
    pub(super) fn new(program: &Program, scope: EntityId) -> Self {
        Self {
            program: program.clone(),
            scope,
        }
    }
}
impl Ord for SourceKey {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self.program.owner_field_0())
            .path
            .parts()
            .identity
            .cmp(&(*other.program.owner_field_0()).path.parts().identity)
            .then_with(|| self.scope.cmp(&other.scope))
    }
}
impl PartialOrd for SourceKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for SourceKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for SourceKey {}
