use crate::authority::mutation::aspect_versions::write_aspect_versions_for_delta;
use crate::authority::mutation::canonical_deltas::{
    authoritative_patch_with_delta_supplements, canonical_delta_for_mutation,
};
use crate::authority::mutation::field_versions::write_field_versions_for_delta;
use crate::authority::mutation::patch_details::{
    patch_detail_for_entity, patch_detail_for_relation, EntityPatchDetailKind,
    RelationPatchDetailKind,
};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchDetail;
use crate::transactions::data::RecordRef;

use super::outcomes::RecordMutation;
use super::{AdjacencyDelta, AdjacencyDeltaKind, MutationEffect, MutationWorkspace};

pub(crate) fn record_publication_effect_for_mutation(
    effect: &mut MutationEffect,
    change: RecordMutation,
    workspace: &mut MutationWorkspace<'_>,
    version_id: VersionId,
) -> Result<(), crate::transactions::data::CommitConflict> {
    let canonical_delta = canonical_delta_for_mutation(&change, workspace)
        .map_err(|error| error.to_commit_conflict())?;
    workspace
        .with_context(|context| {
            write_aspect_versions_for_delta(
                context.state,
                &canonical_delta,
                version_id,
                context.symbols,
            )?;
            write_field_versions_for_delta(
                context.state,
                &canonical_delta,
                version_id,
                context.symbols,
            );
            Ok::<(), crate::authority::mutation::canonical_deltas::CanonicalDeltaError>(())
        })
        .map_err(|error| error.to_commit_conflict())?;

    match change {
        RecordMutation::EntityCreated {
            entity_id,
            authoritative_patch,
            ..
        } => record_entity_publication_fragment(
            effect,
            canonical_delta,
            EntityPatchDetailKind::Created,
            entity_id,
            authoritative_patch,
        ),
        RecordMutation::EntityUpdated {
            entity_id,
            authoritative_patch,
            ..
        } => record_entity_publication_fragment(
            effect,
            canonical_delta,
            EntityPatchDetailKind::Updated,
            entity_id,
            authoritative_patch,
        ),
        RecordMutation::EntityDeleted {
            entity_id,
            authoritative_patch,
            ..
        } => record_entity_publication_fragment(
            effect,
            canonical_delta,
            EntityPatchDetailKind::Deleted,
            entity_id,
            authoritative_patch,
        ),
        RecordMutation::EntityMaterializationSuspended { entity_id, .. } => {
            record_entity_publication_fragment(
                effect,
                canonical_delta,
                EntityPatchDetailKind::MaterializationSuspended,
                entity_id,
                None,
            )
        }
        RecordMutation::EntityRematerialized {
            entity_id,
            authoritative_patch,
            ..
        } => record_entity_publication_fragment(
            effect,
            canonical_delta,
            EntityPatchDetailKind::Rematerialized,
            entity_id,
            authoritative_patch,
        ),
        RecordMutation::RelationCreated {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_patch,
            ..
        } => {
            record_created_adjacency(effect, relation_id, kind_id, source, target);
            record_relation_publication_fragment(
                effect,
                canonical_delta,
                RelationPatchDetailKind::Created,
                relation_id,
                source,
                target,
                authoritative_patch,
            )
        }
        RecordMutation::RelationUpdated {
            relation_id,
            kind_id,
            old_source,
            old_target,
            new_source,
            new_target,
            authoritative_patch,
            ..
        } => {
            record_deleted_adjacency(effect, relation_id, kind_id, old_source, old_target);
            record_created_adjacency(effect, relation_id, kind_id, new_source, new_target);
            record_relation_publication_fragment(
                effect,
                canonical_delta,
                RelationPatchDetailKind::Updated,
                relation_id,
                new_source,
                new_target,
                authoritative_patch,
            )
        }
        RecordMutation::RelationDeleted {
            relation_id,
            kind_id,
            source,
            target,
            ..
        } => {
            record_deleted_adjacency(effect, relation_id, kind_id, source, target);
            record_relation_publication_fragment(
                effect,
                canonical_delta,
                RelationPatchDetailKind::Deleted,
                relation_id,
                source,
                target,
                None,
            )
        }
        RecordMutation::RelationRetainedForAudit {
            relation_id,
            source,
            target,
            ..
        } => record_relation_publication_fragment(
            effect,
            canonical_delta,
            RelationPatchDetailKind::RetainedForAudit,
            relation_id,
            source,
            target,
            None,
        ),
        RecordMutation::RelationMaterializationSuspended {
            relation_id,
            kind_id,
            source,
            target,
            ..
        } => {
            record_deleted_adjacency(effect, relation_id, kind_id, source, target);
            record_relation_publication_fragment(
                effect,
                canonical_delta,
                RelationPatchDetailKind::MaterializationSuspended,
                relation_id,
                source,
                target,
                None,
            )
        }
        RecordMutation::RelationRematerialized {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_patch,
            ..
        } => {
            record_created_adjacency(effect, relation_id, kind_id, source, target);
            record_relation_publication_fragment(
                effect,
                canonical_delta,
                RelationPatchDetailKind::Rematerialized,
                relation_id,
                source,
                target,
                authoritative_patch,
            )
        }
    }
}

fn record_entity_publication_fragment(
    effect: &mut MutationEffect,
    canonical_delta: crate::authority::mutation::CanonicalRecordAspectDelta,
    detail_kind: EntityPatchDetailKind,
    entity_id: crate::identity::data::EntityId,
    authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
) -> Result<(), crate::transactions::data::CommitConflict> {
    effect
        .publication
        .changed_records
        .push(RecordRef::Entity(entity_id));
    record_patch_fragment(
        effect,
        canonical_delta,
        patch_detail_for_entity(detail_kind, entity_id),
        authoritative_patch,
    )
}

fn record_relation_publication_fragment(
    effect: &mut MutationEffect,
    canonical_delta: crate::authority::mutation::CanonicalRecordAspectDelta,
    detail_kind: RelationPatchDetailKind,
    relation_id: crate::identity::data::RelationId,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
    authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
) -> Result<(), crate::transactions::data::CommitConflict> {
    effect
        .publication
        .changed_records
        .push(RecordRef::Relation(relation_id));
    record_patch_fragment(
        effect,
        canonical_delta,
        patch_detail_for_relation(detail_kind, relation_id, source, target),
        authoritative_patch,
    )
}

fn record_created_adjacency(
    effect: &mut MutationEffect,
    relation_id: crate::identity::data::RelationId,
    kind_id: crate::identity::data::KindId,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
) {
    effect.adjacency.deltas.push(AdjacencyDelta {
        relation_id,
        kind_id,
        kind: AdjacencyDeltaKind::Created { source, target },
    });
}

fn record_deleted_adjacency(
    effect: &mut MutationEffect,
    relation_id: crate::identity::data::RelationId,
    kind_id: crate::identity::data::KindId,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
) {
    effect.adjacency.deltas.push(AdjacencyDelta {
        relation_id,
        kind_id,
        kind: AdjacencyDeltaKind::Deleted { source, target },
    });
}

fn record_patch_fragment(
    effect: &mut MutationEffect,
    canonical_delta: crate::authority::mutation::CanonicalRecordAspectDelta,
    detail: PatchDetail,
    authoritative_patch: Option<worth_foundational::facade::AuthoritativeRecordAspectPatch>,
) -> Result<(), crate::transactions::data::CommitConflict> {
    let patch_fragment = match authoritative_patch {
        Some(patch) => {
            let patch = authoritative_patch_with_delta_supplements(&canonical_delta, patch)
                .map_err(|error| error.to_commit_conflict())?;
            let published_patch =
                crate::authority::mutation::canonical_deltas::published_patch_for_delta(
                    &canonical_delta,
                    &patch,
                )
                .map_err(|error| error.to_commit_conflict())?;
            crate::authority::mutation::FoundationalPatchFragment {
                target: canonical_delta.target.clone(),
                structural_change: canonical_delta.structural_change,
                semantic_changes: crate::authority::mutation::canonical_deltas::semantic_changes(
                    &canonical_delta,
                ),
                contains_opaque_aspect: canonical_delta.contains_opaque_aspect,
                detail,
                patch,
                published_patch,
            }
        }
        None => canonical_delta
            .clone()
            .into_foundational_patch_fragment(detail)
            .map_err(|error| error.to_commit_conflict())?,
    };
    effect.publication.canonical_deltas.push(canonical_delta);
    effect.publication.patch_fragments.push(patch_fragment);
    Ok(())
}
