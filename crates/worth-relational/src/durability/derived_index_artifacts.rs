pub(crate) mod captured_decode;
mod checkpoint_delta;
mod decode_budget;
mod generation_decode_quote;

pub(crate) use checkpoint_delta::{CompleteIndexCheckpointView, DerivedIndexCheckpointArtifacts};

use crate::history::data::CanonicalCommitEnvelope;
use crate::indexes::data::DerivedIndexArtifacts;
use crate::runtime::RelationalRuntime;

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

pub(crate) fn validate_checkpoint_generation(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    generation: &crate::indexes::data::DerivedIndexGeneration,
    definition: Option<&crate::indexes::data::DerivedIndexDefinition>,
    history: &crate::runtime::HistorySubsystem,
    generation_frontier: u64,
) -> Result<(), crate::durability::data::DurabilityError> {
    require_generation_affinity(generation)?;
    require_generation_below_frontier(generation.generation_id, generation_frontier)?;
    let definition =
        definition.ok_or_else(|| corrupt("checkpoint index generation has no definition"))?;
    use crate::indexes::data::{DerivedIndexEntries as Entries, DerivedIndexKind as Kind};
    let matching_kind = match (&definition.kind, &generation.entries) {
        (Kind::EntityField { .. }, Entries::EntityField(_))
        | (Kind::RelationField { .. }, Entries::RelationField(_))
        | (Kind::RelatedEntityOrdering { .. }, Entries::RelatedEntityOrdering(_))
        | (Kind::RelationJoin(_), Entries::RelationJoin(_)) => true,
        (
            Kind::EntityField { .. },
            Entries::RelationField(_)
            | Entries::RelatedEntityOrdering(_)
            | Entries::RelationJoin(_),
        )
        | (
            Kind::RelationField { .. },
            Entries::EntityField(_) | Entries::RelatedEntityOrdering(_) | Entries::RelationJoin(_),
        )
        | (
            Kind::RelatedEntityOrdering { .. },
            Entries::EntityField(_) | Entries::RelationField(_) | Entries::RelationJoin(_),
        )
        | (
            Kind::RelationJoin(_),
            Entries::EntityField(_) | Entries::RelationField(_) | Entries::RelatedEntityOrdering(_),
        ) => false,
    };
    if !matching_kind {
        return Err(corrupt(
            "checkpoint index generation kind mismatches definition",
        ));
    }
    if let Entries::RelationJoin(index) = &generation.entries {
        if !index.consistent_legs(lane) {
            return Err(corrupt(
                "checkpoint join leg state is missing or inconsistent",
            ));
        }
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
