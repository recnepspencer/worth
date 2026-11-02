//! Actual immutable Source roster; slots have meaning only with this owner.
use crate::history::{
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial as Denial,
};
use crate::identity::data::EntityId;
use crate::validation::candidate_source::{ManagedIndexSourceProgram, SourcePreparationGroup};
use std::sync::Arc;

#[derive(Debug, Default)]
pub(crate) struct SourceHistoryCatalog {
    programs: Vec<ManagedIndexSourceProgram>,
}
impl SourceHistoryCatalog {
    pub(crate) fn compile(groups: &[SourcePreparationGroup]) -> Self {
        let mut programs = Vec::new();
        for group in groups {
            for program in group.programs() {
                if !programs.contains(program) {
                    programs.push(program.clone());
                }
            }
        }
        Self { programs }
    }
    pub(crate) fn len(&self) -> usize {
        self.programs.len()
    }
    pub(crate) fn program(&self, slot: usize) -> Option<&ManagedIndexSourceProgram> {
        self.programs.get(slot)
    }
    pub(crate) fn has_programs(&self) -> bool {
        !self.programs.is_empty()
    }
    pub(crate) fn slot(&self, program: &ManagedIndexSourceProgram) -> Result<usize, Denial> {
        let slot = self
            .programs
            .iter()
            .position(|actual| actual == program)
            .ok_or(Denial::ForeignManagedSourceProgram)?;
        Ok(slot)
    }
    pub(crate) fn selector(
        self: &Arc<Self>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        let slot = self.slot(program)?;
        if scope.generation_value() == 0 {
            return Err(Denial::UnsupportedManagedSourceSelector);
        }
        Ok(RelationalAuthoredManagedSourceSelector {
            catalog: self.clone(),
            program: slot,
            scope,
        })
    }
}
