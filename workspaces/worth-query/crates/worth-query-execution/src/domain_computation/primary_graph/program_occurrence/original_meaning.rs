//! Installed program meaning retained by a performed correctable occurrence.
use super::*;

pub(in crate::domain_computation) struct OriginalProgramMeaning {
    program: Arc<crate::domain_computation::primary_graph::application_installation::WorthQuerySupportedProgramRecord>,
    interpretation: WorthQueryProgramSupportInterpretation,
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    pub(in crate::domain_computation::primary_graph) fn readmit_original_meaning(
        &self,
        original: &worth_runtime_world::facade::WorldOriginalStateObservation<'_>,
    ) -> Result<
        Arc<OriginalProgramMeaning>,
        super::support_lifecycle::journal::QueryProgramSupportDenial,
    > {
        let program = self
            .native_programs
            .iter()
            .find(|program| program.revision().as_bytes() == original.program_revision().bytes())
            .ok_or(
                super::support_lifecycle::journal::QueryProgramSupportDenial::SupportUnavailable,
            )?;
        let interpretation = self
            .lifecycle
            .readmit_original_interpretation(program.revision(), original)?;
        Ok(Arc::new(OriginalProgramMeaning {
            program: Arc::clone(program),
            interpretation,
        }))
    }
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
    pub(in crate::domain_computation::primary_graph) fn acknowledge_membership(
        &self,
        receiver: &super::support_lifecycle::WorthQueryProgramSupportLifecycle,
        opened: &worth_runtime_world::facade::WorldOpenedPublicationAttempt,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), super::support_lifecycle::journal::QueryProgramSupportDenial> {
        receiver.transfer_original_interpretation(&self.interpretation, opened, request)
    }

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
