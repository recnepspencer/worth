mod build_projection;
mod build_residency;
use build_projection::IndexBuildProjection;
mod generation_publication;
use generation_publication::{
    prepared_generation, publish_index_generations, IndexGenerationPublicationBasis,
};
mod basis_build;
mod build_execution;
mod commit_build;
mod diagnostics;
mod discard;
mod maintenance;
pub(crate) use maintenance::candidate::PreparedCandidateIndexPublication;
#[cfg(test)]
pub(crate) use maintenance::charge_rendezvous::at_next_charges;
mod packet_planning;

use crate::history::data::CommitId;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexBuildOutcome, DerivedIndexBuildRequest,
    DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexGeneration, DerivedIndexGenerationId,
    DerivedIndexId, DerivedIndexPublicationStatus,
};
use crate::runtime::RelationalRuntime;

use self::build_execution::{
    execute_index_inputs, execute_index_packets, index_execution_denial,
    record_index_preparation_strategy_counters, IndexPreparationResult,
};
use self::diagnostics::{
    derived_index_build_artifact_kind, derived_index_build_completed, derived_index_build_scope,
};
use self::packet_planning::{
    choose_index_preparation_strategy, plan_index_packets, plan_index_packets_checked,
    planned_index_definitions,
};
use super::projected_field_values::IndexProjectionSource;
use super::unique_entity_aspect_field_index::{
    rebuild_unique_entity_aspect_field_indexes,
    refresh_unique_entity_aspect_field_index_for_records,
};

pub struct IndexAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

fn failed_build_outcome(
    source_commit_id: CommitId,
    generations: Vec<DerivedIndexGeneration>,
    failed_indexes: Vec<DerivedIndexId>,
    basis_denial: Option<crate::branch::RelationalBranchBasisDenial>,
) -> DerivedIndexBuildOutcome {
    DerivedIndexBuildOutcome {
        source_commit_id,
        generations,
        failed_indexes,
        basis_denial,
        execution_denial: None,
    }
}

impl RelationalRuntime {
    pub fn index_authority(&self) -> IndexAuthority<'_> {
        IndexAuthority::new(self)
    }
}

impl<'runtime> IndexAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub fn register(&self, definition: DerivedIndexDefinition) -> DerivedIndexDefinition {
        self.runtime.indexes.register_definition(definition)
    }

    pub(crate) fn refresh_unique_entity_aspect_field_index_for_records(
        &self,
        changed_records: &[crate::transactions::data::RecordRef],
        basis: &crate::mvcc::PreparedIndexRefreshBasis,
    ) {
        refresh_unique_entity_aspect_field_index_for_records(self.runtime, changed_records, basis);
    }

    pub(crate) fn rebuild_unique_entity_aspect_field_indexes(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::indexes::data::DerivedIndexMaintenanceDenialKind> {
        rebuild_unique_entity_aspect_field_indexes(execution, self.runtime)
    }
}

#[cfg(test)]
pub(crate) use maintenance::last_allocation_bytes;
