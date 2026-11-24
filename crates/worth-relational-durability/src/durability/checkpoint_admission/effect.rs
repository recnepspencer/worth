//! Decode the effect fields from the same authenticated selected commit row.
use worth_relational_branch::internal_port::durability::checkpoint_admission::admitted_commit::CheckpointAdmittedCommit;
use worth_relational_contracts::internal_port::durability::checkpoint_admission::refusal::{RelationalCheckpointAdmissionRefusal as Refusal, projected_stop};
use worth_relational_branch::internal_port::durability::canonical_commit_artifact::decoded_quote::checkpoint_effect_data::CheckpointEffectData;
use crate::durability::canonical_commit_artifact::decoded_quote::decode_checkpoint_effect;
use worth_relational_contracts::internal_port::publication::patch::data::record_structural_change::RecordStructuralChange;
use worth_execution::ExecutionRequest;

pub fn read(
    commit: &CheckpointAdmittedCommit,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CheckpointEffectData, Refusal> {
    let (semantic, creations) = commit
        .original_row()
        .effect_groups()
        .ok_or(Refusal::InvalidProjectedMeaning)?;
    let data = decode_checkpoint_effect(semantic, creations, request).map_err(projected_stop)?;
    if data.semantic.receipt() != &commit.index().commit
        || data.semantic.structural().descriptor_semantics_version
            != commit.schema().descriptor_semantics_version()
        || data
            .creations
            .iter()
            .any(|patch| patch.structural_change != RecordStructuralChange::Created)
    {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    Ok(data)
}
