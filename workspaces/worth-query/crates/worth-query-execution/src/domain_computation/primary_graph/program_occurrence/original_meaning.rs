//! Installed program meaning retained by a performed correctable occurrence.
use super::*;

pub(in crate::domain_computation) struct OriginalProgramMeaning {
    program: Arc<crate::domain_computation::primary_graph::application_installation::WorthQuerySupportedProgramRecord>,
    interpretation: WorthQueryProgramSupportInterpretation,
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    pub(in crate::domain_computation::primary_graph) fn retain_original_meaning(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<Arc<OriginalProgramMeaning>> {
        let program = self
            .native_programs
            .iter()
            .find(|program| program.revision() == revision)?;
        let interpretation = self.retain_interpretation(revision)?;
        Some(Arc::new(OriginalProgramMeaning {
            program: Arc::clone(program),
            interpretation,
        }))
    }
}

impl OriginalProgramMeaning {
    pub(in crate::domain_computation::primary_graph) fn action_inventory(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstalledActionInventory<'_> {
        self.program.installed.action_inventory()
    }
    pub(in crate::domain_computation::primary_graph) fn revision(
        &self,
    ) -> &ApplicationProgramRevision {
        self.program.revision()
    }

    pub(in crate::domain_computation::primary_graph) fn declares_binding(
        &self,
        binding: std::any::TypeId,
    ) -> bool {
        self.program.declares_action(TypeId::from(binding))
            || self.program.requires_output_source(TypeId::from(binding))
    }
}

impl std::fmt::Debug for OriginalProgramMeaning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalProgramMeaning")
            .field("revision", self.revision())
            .finish_non_exhaustive()
    }
}
impl PartialEq for OriginalProgramMeaning {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.program, &other.program)
            && self.interpretation.same_support_as(&other.interpretation)
    }
}
impl Eq for OriginalProgramMeaning {}
