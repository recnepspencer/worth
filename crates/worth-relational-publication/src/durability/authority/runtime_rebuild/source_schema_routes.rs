//! Recovery installs admitted original schema data into the actual publication stream.
use worth_relational_branch::internal_port::branch::transition::original_inputs::{OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation, RetainedBranchTransition};
use worth_relational_contracts::internal_port::branch::transition::sequence::RelationalBranchTransitionSequence;
use worth_relational_durability::internal_port::durability::source_initial_inputs::image::InitialInputsImage;
use worth_relational_durability::internal_port::durability::source_initial_inputs::producer_roster::{OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial};
use crate::runtime::state::subsystems::history::canonical_publication_routes::RelationalCanonicalPublicationRoutes;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

impl worth_relational_durability::internal_port::durability::authority::runtime_rebuild::original_inputs::RecoveredBranchSchemaRoutes for RelationalCanonicalPublicationRoutes {
    fn restore_prefix(
        &self,
        birth: OriginalEmptySchemaObservation,
        installation: Option<OriginalEmptySchemaInstallation>,
        initial: Arc<InitialInputsImage>,
        transitions: Vec<RetainedBranchTransition>,
        memory: Option<ExecutionMemoryReservation>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), OriginalBranchSchemaInputsDenial> {
        self.restore_source_schema_inputs(
            birth,
            installation,
            initial,
            transitions,
            memory,
            request,
        )
    }

    fn capture(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial> {
        self.capture_original_branch_schema_inputs(request)
    }

    fn transition_head(&self) -> RelationalBranchTransitionSequence {
        self.branch_transition_head()
    }

    fn retain_installation(&self, installation: OriginalEmptySchemaInstallation) {
        self.retain_original_empty_installation(installation.before, installation.after);
    }
}
