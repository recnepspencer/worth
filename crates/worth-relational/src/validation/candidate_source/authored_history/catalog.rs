//! Actual immutable Source roster; slots have meaning only with this owner.
use crate::history::{
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial as Denial,
};
use crate::identity::data::EntityId;
use crate::validation::candidate_source::ManagedIndexSourceProgram;
use crate::validation::data::CustomInvariantRegistration;
use std::sync::Arc;

#[derive(Debug, Default)]
pub(crate) struct SourceHistoryCatalog {
    programs: Vec<ManagedIndexSourceProgram>,
    has_membership_programs: bool,
}
impl SourceHistoryCatalog {
    pub(crate) fn compile(registrations: &[CustomInvariantRegistration]) -> Self {
        let mut programs = Vec::new();
        for registration in registrations {
            for program in registration.managed_sources.iter() {
                if !programs.contains(program) {
                    programs.push(program.clone());
                }
            }
        }
        let has_membership_programs = programs.iter().any(membership_profile);
        Self {
            programs,
            has_membership_programs,
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.programs.len()
    }
    pub(crate) fn has_membership_programs(&self) -> bool {
        self.has_membership_programs
    }
    pub(crate) fn membership_slot(
        &self,
        program: &ManagedIndexSourceProgram,
    ) -> Result<usize, Denial> {
        let slot = self
            .programs
            .iter()
            .position(|actual| actual == program)
            .ok_or(Denial::ForeignManagedSourceProgram)?;
        if !membership_profile(program) {
            return Err(Denial::UnsupportedManagedSourceSelector);
        }
        Ok(slot)
    }
    pub(crate) fn selector(
        self: &Arc<Self>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        let slot = self.membership_slot(program)?;
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
pub(super) fn membership_profile(program: &ManagedIndexSourceProgram) -> bool {
    program.fields().is_empty() && program.projection_routes().is_empty()
}
