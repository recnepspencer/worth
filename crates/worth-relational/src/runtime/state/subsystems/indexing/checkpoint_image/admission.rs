//! Validate decoded index data against the admitted definition and history basis.
use crate::durability::data::recovery_errors::DurabilityError;

pub(crate) fn validate_checkpoint_generation(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    generation: &crate::indexes::data::DerivedIndexGeneration,
    definition: Option<&crate::indexes::data::DerivedIndexDefinition>,
    history: &crate::runtime::state::subsystems::history::HistorySubsystem,
    generation_frontier: u64,
) -> Result<(), DurabilityError> {
    require_generation_affinity(generation)?;
    require_generation_below_frontier(generation.generation_id, generation_frontier)?;
    let definition = definition.ok_or_else(|| {
        DurabilityError::corrupt_checkpoint("checkpoint index generation has no definition")
    })?;
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
        return Err(DurabilityError::corrupt_checkpoint(
            "checkpoint index generation kind mismatches definition",
        ));
    }
    if let Entries::RelationJoin(index) = &generation.entries
        && !index.consistent_legs(lane)
    {
        return Err(DurabilityError::corrupt_checkpoint(
            "checkpoint join leg state is missing or inconsistent",
        ));
    }
    let source = history
        .commit_artifact(generation.source_commit_id)
        .ok_or_else(|| {
            DurabilityError::corrupt_checkpoint("checkpoint index source commit is unavailable")
        })?;
    if source.version_id() != generation.applicability.version_id
        || source.schema_version() != generation.applicability.schema_version
    {
        return Err(DurabilityError::corrupt_checkpoint(
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
            .map_err(DurabilityError::corrupt_checkpoint)?;
    }
    Ok(())
}

fn require_generation_below_frontier(
    generation: crate::indexes::data::DerivedIndexGenerationId,
    frontier: u64,
) -> Result<(), DurabilityError> {
    if generation.0 == 0 || generation.0 >= frontier {
        return Err(DurabilityError::corrupt_checkpoint(
            "checkpoint index generation exceeds its eager identity frontier",
        ));
    }
    Ok(())
}

pub(crate) fn require_generation_affinity(
    generation: &crate::indexes::data::DerivedIndexGeneration,
) -> Result<(), DurabilityError> {
    if generation.source_branch_id != generation.applicability.branch_id {
        return Err(DurabilityError::corrupt_checkpoint(
            "derived index generation branch affinity mismatches source",
        ));
    }
    Ok(())
}
