pub mod checkpoint_admission;
pub use checkpoint_admission::{verify_checkpoint_index_bases, CheckpointIndexProjections};

pub mod generation_publication;
use generation_publication::{
    prepared_generation, publish_index_generations, IndexGenerationPublicationBasis,
};

pub mod build_execution;


pub mod maintenance;
pub mod reconstruction_build;

pub use maintenance::PreparedCandidateIndexPublication;
pub use maintenance::PreparedForkIndexSelection;
pub use maintenance::{persistent_map_path_bytes, row_copy_bounds};
pub mod packet_planning;

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::indexes::data::definition::DerivedIndexId;
use worth_relational_branch::internal_port::indexes::data::{
    DerivedIndexBuildOutcome, DerivedIndexBuildRequest, DerivedIndexGeneration,
};

use self::build_execution::{
    execute_index_inputs, execute_index_packets, index_execution_denial, IndexPreparationResult,
};
use self::packet_planning::{
    plan_index_packets, plan_index_packets_checked, planned_index_definitions,
};
use super::projected_field_values::IndexProjectionSource;
pub fn failed_build_outcome(
    source_commit_id: CommitId,
    generations: Vec<DerivedIndexGeneration>,
    failed_indexes: Vec<DerivedIndexId>,
    basis_denial: Option<crate::contracts::branch::basis_denial::RelationalBranchBasisDenial>,
) -> DerivedIndexBuildOutcome {
    DerivedIndexBuildOutcome {
        source_commit_id,
        generations,
        failed_indexes,
        basis_denial,
        execution_denial: None,
    }
}
