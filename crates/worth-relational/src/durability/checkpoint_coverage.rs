/// The last position a checkpoint's retained envelopes include, or zero for a
/// checkpoint of an empty history.
pub(crate) fn checkpoint_coverage(
    checkpoint: &super::data::DurableCheckpoint,
) -> crate::publication::patch::data::PatchStreamPosition {
    use crate::publication::patch::data::PatchStreamPosition;
    if let Some(directory) = &checkpoint.directory {
        return directory.manifest.covered_position;
    }
    checkpoint
        .envelopes
        .iter()
        .map(crate::history::data::PositionedCanonicalCommit::position)
        .max()
        .unwrap_or(PatchStreamPosition(0))
}
