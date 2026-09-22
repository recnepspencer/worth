use crate::history::data::CanonicalCommitEnvelope;
use crate::indexes::data::DerivedIndexArtifacts;
use crate::runtime::RelationalRuntime;

pub(crate) fn checkpoint_derived_index_artifacts(
    runtime: &RelationalRuntime,
) -> DerivedIndexArtifacts {
    DerivedIndexArtifacts::new(runtime.index_access().generations_snapshot())
}

pub(crate) fn restore_checkpoint_derived_index_artifacts(
    indexes: &mut crate::runtime::IndexingState,
    artifacts: &DerivedIndexArtifacts,
) {
    for generation in artifacts.generations() {
        indexes.restore_generation(generation.clone());
    }
}

pub(crate) fn apply_envelope_derived_index_artifacts(
    runtime: &RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
) {
    if envelope.derived_index_artifacts().is_empty() {
        return;
    }

    for generation in envelope.derived_index_artifacts().generations() {
        runtime.indexes.restore_generation(generation.clone());
    }
}
