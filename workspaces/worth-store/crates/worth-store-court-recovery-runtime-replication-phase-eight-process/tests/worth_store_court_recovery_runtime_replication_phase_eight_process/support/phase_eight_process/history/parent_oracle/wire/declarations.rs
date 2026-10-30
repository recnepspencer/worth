pub(crate) fn observe_checkpoint(
    bytes: &[u8],
) -> Option<crate::phase_eight_process::history::parent_oracle::artifact_facts::CheckpointFacts> {
    crate::phase_eight_process::history::parent_oracle::wire::checkpoint::observe_checkpoint(bytes)
}

pub(crate) fn validated_checkpoint_binding_payloads(bytes: &[u8]) -> Option<Vec<&[u8]>> {
    crate::phase_eight_process::history::parent_oracle::wire::checkpoint::validated_binding_payloads(
        bytes,
    )
}

pub(crate) fn observe_wal(
    bytes: &[u8],
) -> crate::phase_eight_process::history::parent_oracle::artifact_facts::ArtifactFacts {
    crate::phase_eight_process::history::parent_oracle::wire::wal::observe_wal(bytes)
}
