pub(crate) mod checkpoint_admission;
pub(crate) use checkpoint_admission::{verify_checkpoint_index_bases, CheckpointIndexProjections};
mod build_diagnostics;
mod build_projection;
mod build_residency;
mod generation_publication;
use generation_publication::{
    prepared_generation, publish_index_generations, IndexGenerationPublicationBasis,
};
mod basis_build;
mod build_execution;
mod commit_build;
mod diagnostics;
mod discard;
mod live_access;
pub(crate) mod maintenance;
mod recovery_build;
#[cfg(test)]
pub(crate) use maintenance::charge_rendezvous::at_next_charges;
pub(crate) use maintenance::PreparedCandidateIndexPublication;
pub(crate) use maintenance::PreparedForkIndexSelection;
pub(crate) use maintenance::{persistent_map_path_bytes, row_copy_bounds};
mod packet_planning;

use crate::history::data::CommitId;
use crate::indexes::data::{
    DerivedIndexBuildOutcome, DerivedIndexBuildRequest, DerivedIndexGeneration, DerivedIndexId,
};

use self::build_execution::{
    execute_index_inputs, execute_index_packets, index_execution_denial, IndexPreparationResult,
};
use self::diagnostics::{
    derived_index_build_artifact_kind, derived_index_build_completed, derived_index_build_scope,
};
use self::packet_planning::{
    plan_index_packets, plan_index_packets_checked, planned_index_definitions,
};
use super::projected_field_values::IndexProjectionSource;
fn failed_build_outcome(
    source_commit_id: CommitId,
    generations: Vec<DerivedIndexGeneration>,
    failed_indexes: Vec<DerivedIndexId>,
    basis_denial: Option<crate::branch::basis_denial::RelationalBranchBasisDenial>,
) -> DerivedIndexBuildOutcome {
    DerivedIndexBuildOutcome {
        source_commit_id,
        generations,
        failed_indexes,
        basis_denial,
        execution_denial: None,
    }
}

#[cfg(test)]
pub(crate) use maintenance::last_allocation_bytes;
