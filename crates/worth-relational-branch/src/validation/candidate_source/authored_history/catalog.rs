//! Actual immutable Source roster; slots have meaning only with this owner.
use super::catalog_funding::CustomInvariantCompilationFunding;
use crate::history::access::relevant_writes::RelationalAuthoredManagedSourceSelector;
use crate::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct SourceHistoryCatalog {
    programs: Vec<ManagedIndexSourceProgram>,
    _funding: Option<Arc<CustomInvariantCompilationFunding>>,
}
impl SourceHistoryCatalog {
    pub fn from_programs(
        programs: Vec<ManagedIndexSourceProgram>,
        funding: Option<Arc<CustomInvariantCompilationFunding>>,
    ) -> Self {
        Self {
            programs,
            _funding: funding,
        }
    }
    pub fn len(&self) -> usize {
        self.programs.len()
    }
    pub fn program(&self, slot: usize) -> Option<&ManagedIndexSourceProgram> {
        self.programs.get(slot)
    }
    pub fn has_programs(&self) -> bool {
        !self.programs.is_empty()
    }
    pub fn slot(&self, program: &ManagedIndexSourceProgram) -> Result<usize, Denial> {
        let slot = self
            .programs
            .iter()
            .position(|actual| actual == program)
            .ok_or(Denial::ForeignManagedSourceProgram)?;
        Ok(slot)
    }
    pub fn selector(
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
