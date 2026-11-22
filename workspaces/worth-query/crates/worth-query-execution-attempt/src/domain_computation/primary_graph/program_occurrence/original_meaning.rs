//! Installed program meaning retained by a performed correctable occurrence.
use std::any::TypeId;
use std::sync::Arc;

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
pub use worth_query_execution_lineage::facade::primary_graph::query_obligations::history::WorthQueryProgramSupportInterpretation;

use crate::ports::SupportedProgramRecordPort;

pub struct OriginalProgramMeaning {
    program: Arc<dyn SupportedProgramRecordPort>,
    interpretation: WorthQueryProgramSupportInterpretation,
}

impl OriginalProgramMeaning {
    pub fn from_parts(
        program: Arc<dyn SupportedProgramRecordPort>,
        interpretation: WorthQueryProgramSupportInterpretation,
    ) -> Self {
        Self {
            program,
            interpretation,
        }
    }

    pub fn acknowledge_membership(
        &self,
        receiver: &crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        opened: &worth_runtime_world::facade::WorldOpenedPublicationAttempt,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
    > {
        receiver.transfer_original_interpretation(&self.interpretation, opened, request)
    }

    pub fn action_inventory(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstalledActionInventory<'_> {
        self.program.action_inventory()
    }

    pub fn revision(&self) -> &ApplicationProgramRevision {
        self.program.revision()
    }

    pub fn declares_binding(&self, binding: TypeId) -> bool {
        self.program.declares_binding(binding)
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
