pub(crate) mod captured_decode;
mod checkpoint_delta;
mod decode_budget;
mod frozen_checkpoint;
mod funding;
pub(crate) use frozen_checkpoint::FrozenIndexCheckpoint;
#[path = "../runtime/state/subsystems/indexing/cold_checkpoint.rs"]
pub(crate) mod cold_checkpoint;
mod generation_decode_quote;
pub(crate) mod original_capture;
mod projection_verification;

pub(crate) use checkpoint_delta::{CompleteIndexCheckpointView, DerivedIndexCheckpointArtifacts};

use crate::history::data::CanonicalCommitEnvelope;
use crate::indexes::data::DerivedIndexArtifacts;
use crate::runtime::state::subsystems::indexing::checkpoint_image::admission::{
    require_generation_affinity, validate_checkpoint_generation,
};
use crate::runtime::RelationalRuntime;

pub(crate) fn checkpoint_index_projections(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    runtime: &RelationalRuntime,
) -> crate::indexes::authority::CheckpointIndexProjections {
    crate::indexes::authority::CheckpointIndexProjections::from_context(
        runtime.checkpoint_projection_context(lane),
    )
}

pub(crate) fn restore_checkpoint_derived_index_artifacts(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    indexes: &mut crate::runtime::IndexingState,
    history: &crate::runtime::HistorySubsystem,
    legacy: &DerivedIndexArtifacts,
    checkpoint: Option<&DerivedIndexCheckpointArtifacts>,
    generation_frontier: u64,
    work: &mut crate::durability::data::CheckpointRestoreWork,
) -> Result<(), crate::durability::data::DurabilityError> {
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

fn corrupt(detail: &str) -> crate::durability::data::DurabilityError {
    crate::durability::data::DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
        detail,
    )
}

pub(crate) struct AdmittedEnvelopeIndexArtifacts<'a>(&'a DerivedIndexArtifacts);

pub(crate) fn admit_envelope_derived_index_artifacts(
    envelope: &CanonicalCommitEnvelope,
) -> Result<AdmittedEnvelopeIndexArtifacts<'_>, crate::durability::data::DurabilityError> {
    for generation in envelope.derived_index_artifacts().generations() {
        require_generation_affinity(generation)?;
    }
    Ok(AdmittedEnvelopeIndexArtifacts(
        envelope.derived_index_artifacts(),
    ))
}

impl AdmittedEnvelopeIndexArtifacts<'_> {
    pub(crate) fn install(self, runtime: &RelationalRuntime) {
        for generation in self.0.generations() {
            runtime.indexes.restore_generation(generation.clone());
        }
    }
}
