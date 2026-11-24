/// The last position a checkpoint's retained envelopes include, or zero for a
/// checkpoint of an empty history.
pub(crate) fn checkpoint_coverage(
    checkpoint: &super::data::DurableCheckpoint,
) -> worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition{
    use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;
    if let Some(directory) = &checkpoint.directory {
        return directory.manifest.covered_position;
    }
    checkpoint
        .envelopes
        .iter()
        .map(worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit::position)
        .max()
        .unwrap_or(PatchStreamPosition(0))
}
