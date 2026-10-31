//! Bounded agreement of descriptive selection provenance with its envelope.
use super::RelationalCommitArtifactDenial;
use crate::branch::{relational_branch_observation, CANONICAL_RUNTIME_INSTANCE_ID};
use crate::history::data::{CanonicalCommitEnvelope, RelationalDescriptiveTouch};
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;
use worth_foundational::FoundationalBranchTarget;

/// Internal meaning that a framed selection artifact failed to establish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRetainedSelectionArtifactDenial {
    SourceDescriptor,
    EntityIdentity,
    SchemaAgreement,
    TargetPredecessor,
    UnsupportedProfile,
    SelectedRecordMeaning,
}

pub(super) fn validate(
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), RelationalCommitArtifactDenial> {
    validate_selection(envelope).map_err(RelationalCommitArtifactDenial::RetainedEntitySelection)
}

fn validate_selection(
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), RelationalRetainedSelectionArtifactDenial> {
    use RelationalRetainedSelectionArtifactDenial as Denial;
    let Some(selection) = envelope.authority_kind.retained_entity_selection() else {
        return Ok(());
    };
    let source = selection.source_head();
    if source.runtime_instance_id() != CANONICAL_RUNTIME_INSTANCE_ID
        || source.selected_commit_id() == 0
        || source.version_id() == 0
        || source.selected_commit_id() == envelope.commit.commit_id.0
        || relational_branch_observation(
            CANONICAL_RUNTIME_INSTANCE_ID,
            &selection.source_branch().0,
            FoundationalBranchTarget::basis(source.clone()),
            selection.source_generation(),
        )
        .is_err()
        || crate::history::RelationalCommitParentage::from_ordered(
            source
                .parent_commit_ids()
                .iter()
                .copied()
                .map(crate::history::data::CommitId)
                .collect(),
        )
        .is_err()
    {
        return Err(Denial::SourceDescriptor);
    }
    if selection.entity().generation.is_zero() {
        return Err(Denial::EntityIdentity);
    }
    if !envelope
        .schema_authority
        .entity_kinds
        .iter()
        .any(|kind| kind.kind_id == selection.kind())
        || source.roots().schema_root()
            != &crate::schema::data::schema_authority_snapshot_digest_bytes(
                &envelope.schema_authority,
            )
    {
        return Err(Denial::SchemaAgreement);
    }
    if envelope.strategy_artifacts.is_some()
        || envelope.merge_execution_authority.is_some()
        || !envelope.merge_parent_branches.is_empty()
        || !envelope.merge_base_commits.is_empty()
        || envelope.schema_transition.is_some()
        || envelope.schema_continuation_descriptor.is_some()
        || envelope.schema_reconciliation_descriptor.is_some()
        || envelope.required_output_origin.is_some()
        || envelope.required_output_materialization.is_some()
        || !envelope.required_output_codec_retirements.is_empty()
    {
        return Err(Denial::UnsupportedProfile);
    }
    let before = envelope
        .branch_cell_checkpoint
        .as_ref()
        .ok_or(Denial::TargetPredecessor)?;
    if before.runtime_instance_id != CANONICAL_RUNTIME_INSTANCE_ID
        || before.branch_id != envelope.branch_context
        || before.in_canonical_frame().as_ref() != Ok(before)
        || envelope.commit.branch_id != envelope.branch_context
    {
        return Err(Denial::TargetPredecessor);
    }
    let FoundationalBranchTarget::Basis(target) = before.observation.target() else {
        return Err(Denial::TargetPredecessor);
    };
    if envelope.commit.parents.as_slice()
        != [crate::history::data::CommitId(target.selected_commit_id())]
        || target.selected_commit_id() == envelope.commit.commit_id.0
        || target.roots().schema_root() != source.roots().schema_root()
    {
        return Err(Denial::TargetPredecessor);
    }
    let selected = RecordRef::Entity(selection.entity());
    // Equal payloads can legitimately have no field patch. Any present patch
    // must still describe a non-lifecycle update of this exact full-GEN record.
    if envelope
        .patch
        .authoritative_record_patches
        .iter()
        .any(|patch| {
            patch.target == selected
                && (patch.structural_change != RecordStructuralChange::Updated
                    || patch.relation_birth.is_some())
        })
        || envelope
            .descriptive_touches()
            .exact_touches()
            .into_iter()
            .flatten()
            .any(|touch| match touch {
                RelationalDescriptiveTouch::EntityLifecycle { entity, .. } => {
                    *entity == selection.entity()
                }
                RelationalDescriptiveTouch::FieldRevision { record, kind, .. } => {
                    record == &selected && *kind != selection.kind()
                }
                _ => false,
            })
    {
        return Err(Denial::SelectedRecordMeaning);
    }
    Ok(())
}
