//! The verified occurrence-program gate retains source interpretation before effects.
use super::*;
use crate::domain_computation::primary_graph::{
    program_occurrence::WorthQueryInstalledProgramSupport, WorthQueryApplicationCommitDenial,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn retain_original_program(
        mut self,
        support: &WorthQueryInstalledProgramSupport<Schema>,
        revision: &ApplicationProgramRevision,
    ) -> Result<Self, WorthQueryApplicationCommitDenial> {
        let admission = &mut self.read_set.admission;
        if !admission
            .allowed_graph_contract()
            .aftermath()
            .is_some_and(|contract| contract.correction_disclosure().is_some())
        {
            return Ok(self);
        }
        if admission.original_correction_disclosure().is_none() {
            return Err(WorthQueryApplicationCommitDenial::mutation_input_mismatch());
        }
        let binding = admission
            .original_mutation_binding()
            .ok_or_else(WorthQueryApplicationCommitDenial::mutation_binding_mismatch)?;
        let meaning = support
            .retain_original_meaning(revision)
            .ok_or_else(WorthQueryApplicationCommitDenial::application_program_required)?;
        if !meaning.declares_binding(binding) {
            return Err(WorthQueryApplicationCommitDenial::mutation_binding_mismatch());
        }
        admission.retain_original_program(meaning);
        Ok(self)
    }
}
