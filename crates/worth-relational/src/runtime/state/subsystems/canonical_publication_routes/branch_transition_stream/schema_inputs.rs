//! Original schema DATA owned by the producer, independently of live roots.

use std::sync::Arc;

use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_foundational::FoundationalBranchReferenceGeneration;

use crate::branch::{
    PositionedBranchTransition, RelationalBranchLifecyclePosture,
    RelationalBranchRootSchemaAuthority, RelationalBranchTransitionSequence,
    RelationalBranchVersion,
};

use super::super::RelationalCanonicalPublicationRoutes;

/// The branch name comes from the same source's existing main-branch binding.
/// Empty live branches cannot be forked, and main cannot be deleted.
#[derive(Clone, Debug)]
pub(crate) struct OriginalEmptySchemaObservation {
    pub(crate) generation: FoundationalBranchReferenceGeneration,
    pub(crate) truth_version: RelationalBranchVersion,
    pub(crate) lifecycle: RelationalBranchLifecyclePosture,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
}

#[derive(Clone, Debug)]
pub(crate) struct OriginalEmptySchemaInstallation {
    pub(crate) before: OriginalEmptySchemaObservation,
    pub(crate) after: OriginalEmptySchemaObservation,
    /// Initial installation is possible only before the first canonical write.
    pub(crate) follows_transition: RelationalBranchTransitionSequence,
}

/// The actual performed transition supplies its own branch name and C/T axes.
#[derive(Clone, Debug)]
pub(crate) struct RetainedBranchTransition {
    pub(crate) positioned: Arc<PositionedBranchTransition>,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
}

/// An independently admitted capture roster. Cloning existing schema/record
/// Arcs does not copy their backing or retroactively fund ordinary producers.
#[derive(Debug)]
pub(crate) struct OriginalBranchSchemaInputs {
    pub(crate) birth: Option<OriginalEmptySchemaObservation>,
    pub(crate) installation: Option<OriginalEmptySchemaInstallation>,
    pub(crate) accepted_initial_inputs:
        Option<Arc<crate::durability::source_initial_inputs::image::InitialInputsImage>>,
    transitions: Vec<RetainedBranchTransition>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl OriginalBranchSchemaInputs {
    pub(crate) fn transitions(&self) -> &[RetainedBranchTransition] {
        &self.transitions
    }
}

impl OriginalEmptySchemaObservation {
    pub(crate) fn from_empty_cell(
        cell: &crate::branch::RelationalBranchReferenceCell,
    ) -> Option<Self> {
        let (generation, truth_version, lifecycle, schema) = cell.retained_empty_schema_data()?;
        Some(Self {
            generation,
            truth_version,
            lifecycle,
            schema,
        })
    }
}

#[derive(Debug)]
pub(crate) enum OriginalBranchSchemaInputsDenial {
    SizeOverflow,
    Request(WorkCeilingDenial),
    OriginalPrefixMismatch,
}

pub(super) fn roster_bytes(capacity: usize) -> Option<u64> {
    let layout = std::alloc::Layout::array::<RetainedBranchTransition>(capacity).ok()?;
    u64::try_from(layout.size()).ok()
}

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
    pub(crate) fn retain_original_empty_installation(
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
    pub(crate) fn capture_original_branch_schema_inputs(
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
        let bytes = roster_bytes(count).ok_or(OriginalBranchSchemaInputsDenial::SizeOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(OriginalBranchSchemaInputsDenial::Request)?;
        let mut transitions = Vec::with_capacity(count);
        for entry in &state.log[..count] {
            request
                .consult()
                .map_err(OriginalBranchSchemaInputsDenial::Request)?;
            transitions.push(entry.clone());
        }
        Ok(OriginalBranchSchemaInputs {
            birth: state.birth.clone(),
            installation: state.installation.clone(),
            accepted_initial_inputs: state.initial_inputs.clone(),
            transitions,
            _memory: memory,
        })
    }

    pub(crate) fn retain_accepted_initial_inputs(
        &self,
        inputs: Arc<crate::durability::source_initial_inputs::image::InitialInputsImage>,
    ) {
        self.branch_transitions.state().initial_inputs = Some(inputs);
    }
}

#[cfg(test)]
#[path = "schema_inputs/tests.rs"]
mod tests;
