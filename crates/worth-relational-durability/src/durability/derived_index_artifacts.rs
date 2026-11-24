use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
pub mod captured_decode;
pub mod checkpoint_delta;
pub mod decode_budget;
pub mod frozen_checkpoint;
pub mod funding;
pub use frozen_checkpoint::FrozenIndexCheckpoint;
pub mod cold_checkpoint;
pub mod generation_decode_quote;
pub mod original_capture;
pub mod projection_verification;

pub(crate) use checkpoint_delta::CompleteIndexCheckpointView;
pub use checkpoint_delta::DerivedIndexCheckpointArtifacts;

use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::indexes::data::DerivedIndexArtifacts;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_image::admission::{require_generation_affinity, validate_checkpoint_generation};
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingSubsystem;
use branch::visibility::materialization::read_records::projection::checkpoint_context::CheckpointProjectionContext;
use branch::visibility::materialization::read_records::projection::creation::ProjectionInputs;

pub fn checkpoint_index_projections<'a, Source>(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    source: &'a Source,
) -> worth_relational_indexes::internal_port::indexes::authority::checkpoint_admission::CheckpointIndexProjections
where
    &'a Source: Into<ProjectionInputs<'a>>,
{
    worth_relational_indexes::internal_port::indexes::authority::checkpoint_admission::CheckpointIndexProjections::from_context(
        CheckpointProjectionContext::from_inputs(lane, source.into()),
    )
}

pub fn restore_checkpoint_derived_index_artifacts(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    indexes: &mut worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingState,
    history: &branch::runtime::state::subsystems::history::HistorySubsystem,
    legacy: &DerivedIndexArtifacts,
    checkpoint: Option<&DerivedIndexCheckpointArtifacts>,
    generation_frontier: u64,
    work: &mut crate::durability::data::CheckpointRestoreWork,
) -> Result<(), contracts::durability::data::recovery_errors::DurabilityError> {
    if generation_frontier == 0 {
        return Err(corrupt("checkpoint index generation frontier is zero"));
    }
    let generations = if let Some(checkpoint) = checkpoint {
        if !legacy.is_empty() {
            return Err(corrupt("checkpoint mixes legacy and delta index artifacts"));
        }
        checkpoint.readmit(lane)?
    } else {
        legacy.generations().to_vec()
    };
    for generation in &generations {
        validate_checkpoint_generation(
            lane,
            generation,
            indexes.definition(generation.index_id).map(AsRef::as_ref),
            history,
            generation_frontier,
        )?;
    }
    for generation in generations {
        indexes.restore_generation(generation);
        work.index_generations_readmitted += 1;
    }
    if let Some(checkpoint) = checkpoint {
        for basis in checkpoint.bases() {
            history
                .validate_recovered_scoped_index_basis(
                    &basis.branch,
                    basis.commit,
                    basis.version,
                    basis.schema,
                )
                .map_err(|detail| corrupt(&detail))?;
            indexes
                .restore_basis_selection(lane, basis)
                .map_err(corrupt)?;
        }
    }
    Ok(())
}

fn corrupt(detail: &str) -> contracts::durability::data::recovery_errors::DurabilityError {
    contracts::durability::data::recovery_errors::DurabilityError::new(
        contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
        detail,
    )
}

pub struct AdmittedEnvelopeIndexArtifacts<'a>(&'a DerivedIndexArtifacts);

pub fn admit_envelope_derived_index_artifacts(
    envelope: &CanonicalCommitEnvelope,
) -> Result<
    AdmittedEnvelopeIndexArtifacts<'_>,
    contracts::durability::data::recovery_errors::DurabilityError,
> {
    for generation in envelope.derived_index_artifacts().generations() {
        require_generation_affinity(generation)?;
    }
    Ok(AdmittedEnvelopeIndexArtifacts(
        envelope.derived_index_artifacts(),
    ))
}

impl AdmittedEnvelopeIndexArtifacts<'_> {
    pub fn install<'a, Owner>(self, owner: &'a Owner)
    where
        &'a Owner: Into<&'a IndexingSubsystem>,
    {
        let indexes = owner.into();
        for generation in self.0.generations() {
            indexes.restore_generation(generation.clone());
        }
    }
}
