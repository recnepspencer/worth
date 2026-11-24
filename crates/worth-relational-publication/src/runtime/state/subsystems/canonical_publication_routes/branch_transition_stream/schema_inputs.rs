//! Original schema DATA owned by the producer, independently of live roots.

use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::transition::original_inputs::{
    OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation,
};

use super::super::RelationalCanonicalPublicationRoutes;
use worth_relational_durability::internal_port::durability::source_initial_inputs::producer_roster::{OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial, capture_producer_roster};

impl RelationalCanonicalPublicationRoutes {
    /// A freshly constructed recovery receiver does not own its source's C0.
    /// The isolated receiver still owns its construction root when this runs;
    /// dropping this schema share is nonfinal. Same-source continuations never
    /// call this operation.
    pub(crate) fn discard_unobserved_construction_schema_inputs_for_recovery(&self) {
        let mut state = self.branch_transitions.state();
        assert!(
            state.log.is_empty(),
            "recovery receiver has no performed transitions"
        );
        assert!(
            state.installation.is_none(),
            "recovery receiver has no initial installation"
        );
        state.birth = None;
    }

    /// Construction installs C0 before publishing the fresh runtime. These are
    /// fixed inline fields in an already ordinary, unqualified stream owner.
    pub(crate) fn retain_original_empty_birth(&self, birth: OriginalEmptySchemaObservation) {
        let mut state = self.branch_transitions.state();
        assert!(
            state.birth.is_none(),
            "C0 schema is installed once at birth"
        );
        state.birth = Some(birth);
    }

    /// Called only after all installation preparation succeeds and immediately
    /// before replacing Empty cells under the existing configuration epoch.
    pub fn retain_original_empty_installation(
        &self,
        before: OriginalEmptySchemaObservation,
        after: OriginalEmptySchemaObservation,
    ) {
        let mut state = self.branch_transitions.state();
        assert!(
            state.installation.is_none(),
            "initial schema installation seals its one legal transition"
        );
        state.installation = Some(OriginalEmptySchemaInstallation {
            before,
            after,
            follows_transition: state.head,
        });
    }

    /// The caller freezes the source's existing capture cut. All covered
    /// producer originals remain available; ordinary suffix coverage is not
    /// evidence that a Complete consumer has acquired these inputs.
    pub fn capture_original_branch_schema_inputs(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial> {
        let state = self.branch_transitions.state();
        request
            .consult()
            .map_err(OriginalBranchSchemaInputsDenial::Request)?;
        let count = state
            .log
            .partition_point(|entry| entry.positioned.sequence() <= state.head);
        capture_producer_roster(
            state.birth.as_ref(),
            state.installation.as_ref(),
            state.initial_inputs.as_ref(),
            &state.log[..count],
            request,
        )
    }

    /// Immutable source lineage follows the same original-input capture epoch.
    /// The seal and descriptions retain no live World port or Home lease.
    pub fn retain_home_world_selection(
        &self,
        source: Arc<
            worth_proof::VerifiedHomeWorldSelection<
                Arc<worth_relational_durability::internal_port::durability::home_world_catalog::RelationalWorldSourceCatalog>,
            >,
        >,
    ) {
        self.branch_transitions.state().home_world_selection = Some(source);
    }

    pub fn retain_accepted_initial_inputs(
        &self,
        inputs: Arc<worth_relational_durability::internal_port::durability::source_initial_inputs::image::InitialInputsImage>,
    ) {
        self.branch_transitions.state().initial_inputs = Some(inputs);
    }
}
