mod checkpoint_delta;

pub(crate) use checkpoint_delta::DerivedIndexCheckpointArtifacts;

use crate::history::data::CanonicalCommitEnvelope;
use crate::indexes::data::DerivedIndexArtifacts;
use crate::runtime::RelationalRuntime;

pub(crate) fn checkpoint_derived_index_artifacts(
    runtime: &RelationalRuntime,
    retained: &crate::history::retention::RetainedIndexRoots,
) -> Result<DerivedIndexCheckpointArtifacts, crate::durability::data::DurabilityError> {
    DerivedIndexCheckpointArtifacts::capture(runtime.indexes.retained_generations(retained))
}

pub(crate) fn restore_checkpoint_derived_index_artifacts(
    indexes: &mut crate::runtime::IndexingState,
    legacy: &DerivedIndexArtifacts,
    checkpoint: Option<&DerivedIndexCheckpointArtifacts>,
    envelopes: &[crate::history::data::PositionedCanonicalCommit],
) -> Result<(), crate::durability::data::DurabilityError> {
    if let Some(checkpoint) = checkpoint {
        if !legacy.is_empty() {
            return Err(corrupt("checkpoint mixes legacy and delta index artifacts"));
        }
        let source_commits = envelopes
            .iter()
            .map(|envelope| (envelope.commit.commit_id, envelope))
            .collect::<std::collections::BTreeMap<_, _>>();
        if source_commits.len() != envelopes.len() {
            return Err(corrupt("checkpoint index source commits are duplicated"));
        }
        for generation in checkpoint.readmit()? {
            let definition = indexes
                .definitions
                .get(&generation.index_id)
                .ok_or_else(|| corrupt("checkpoint index generation has no definition"))?;
            use crate::indexes::data::{DerivedIndexEntries as Entries, DerivedIndexKind as Kind};
            let matching_kind = matches!(
                (&definition.kind, &generation.entries),
                (Kind::EntityField { .. }, Entries::EntityField(_))
                    | (Kind::RelationField { .. }, Entries::RelationField(_))
                    | (
                        Kind::RelatedEntityOrdering { .. },
                        Entries::RelatedEntityOrdering(_)
                    )
                    | (Kind::RelationJoin(_), Entries::RelationJoin(_))
            );
            if !matching_kind {
                return Err(corrupt(
                    "checkpoint index generation kind mismatches definition",
                ));
            }
            let source = source_commits
                .get(&generation.source_commit_id)
                .ok_or_else(|| corrupt("checkpoint index source commit is unavailable"))?;
            if source.commit.version_id != generation.applicability.version_id
                || source.envelope().schema_version != generation.applicability.schema_version
            {
                return Err(corrupt(
                    "checkpoint index generation basis mismatches commit",
                ));
            }
            indexes.restore_generation(generation);
        }
    } else {
        for generation in legacy.generations() {
            indexes.restore_generation(generation.clone());
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

pub(crate) fn apply_envelope_derived_index_artifacts(
    runtime: &RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
) {
    for generation in envelope.derived_index_artifacts().generations() {
        runtime.indexes.restore_generation(generation.clone());
    }
}
