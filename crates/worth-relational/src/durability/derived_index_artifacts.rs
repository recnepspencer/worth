mod checkpoint_delta;

pub(crate) use checkpoint_delta::DerivedIndexCheckpointArtifacts;

use crate::history::data::CanonicalCommitEnvelope;
use crate::indexes::data::DerivedIndexArtifacts;
use crate::runtime::RelationalRuntime;

pub(crate) fn restore_checkpoint_derived_index_artifacts(
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
        checkpoint.readmit()?
    } else {
        legacy.generations().to_vec()
    };
    for generation in &generations {
        require_generation_affinity(generation)?;
        require_generation_below_frontier(generation.generation_id, generation_frontier)?;
        let definition = indexes
            .definition(generation.index_id)
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
        let source = history
            .commit_artifact(generation.source_commit_id)
            .ok_or_else(|| corrupt("checkpoint index source commit is unavailable"))?;
        if source.version_id() != generation.applicability.version_id
            || source.schema_version() != generation.applicability.schema_version
        {
            return Err(corrupt(
                "checkpoint index generation basis mismatches commit",
            ));
        }
        if definition.branch_scoped {
            history
                .validate_recovered_scoped_index_basis(
                    &generation.source_branch_id,
                    generation.source_commit_id,
                    generation.applicability.version_id,
                    generation.applicability.schema_version,
                )
                .map_err(|detail| corrupt(&detail))?;
        }
    }
    for generation in generations {
        indexes.restore_generation(generation);
        work.index_generations_readmitted += 1;
    }
    Ok(())
}

fn require_generation_below_frontier(
    generation: crate::indexes::data::DerivedIndexGenerationId,
    frontier: u64,
) -> Result<(), crate::durability::data::DurabilityError> {
    if generation.0 == 0 || generation.0 >= frontier {
        return Err(corrupt(
            "checkpoint index generation exceeds its eager identity frontier",
        ));
    }
    Ok(())
}

fn corrupt(detail: &str) -> crate::durability::data::DurabilityError {
    crate::durability::data::DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
        detail,
    )
}

fn require_generation_affinity(
    generation: &crate::indexes::data::DerivedIndexGeneration,
) -> Result<(), crate::durability::data::DurabilityError> {
    if generation.source_branch_id != generation.applicability.branch_id {
        return Err(corrupt(
            "derived index generation branch affinity mismatches source",
        ));
    }
    Ok(())
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
