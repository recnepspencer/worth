use crate::identity::data::VersionId;
use crate::merge::data::{
    MergeVisibilityEvidence, MergeVisibilityEvidenceKind, MergeVisibilityState, VisibleMergeRecord,
    VisibleMergeRecordKind,
};
use crate::storage::data::RecordLifecycleState;
use crate::transactions::data::primitives::RecordRef;
use crate::visibility::snapshot_read_view::RelationalReadView;

pub(super) fn source_record_visibility_evidence(
    record: &VisibleMergeRecord,
) -> MergeVisibilityEvidence {
    match record.record_kind {
        VisibleMergeRecordKind::Entity => embedded_visibility_evidence(
            record.record_ref.clone(),
            MergeVisibilityEvidenceKind::SourceEmbeddedSurface,
            record.source_entity.as_ref().map(|entity| entity.lifecycle),
            record
                .source_entity
                .as_ref()
                .map(|entity| entity.created_at_version),
            record
                .source_entity
                .as_ref()
                .and_then(|entity| entity.retired_at_version),
        ),
        VisibleMergeRecordKind::Relation => embedded_visibility_evidence(
            record.record_ref.clone(),
            MergeVisibilityEvidenceKind::SourceEmbeddedSurface,
            record
                .source_relation
                .as_ref()
                .map(|relation| relation.lifecycle),
            record
                .source_relation
                .as_ref()
                .map(|relation| relation.created_at_version),
            record
                .source_relation
                .as_ref()
                .and_then(|relation| relation.retired_at_version),
        ),
    }
}

pub(super) fn target_record_visibility_evidence(
    record: &VisibleMergeRecord,
    candidate_target_record: Option<&RecordRef>,
    target_view: &RelationalReadView,
) -> MergeVisibilityEvidence {
    if let Some(target_record) = candidate_target_record {
        let resolved_target = view_record_visibility_metadata(target_view, target_record);
        return MergeVisibilityEvidence {
            observed_record: target_record.clone(),
            kind: MergeVisibilityEvidenceKind::TargetCandidateViewLookup,
            state: resolved_target
                .as_ref()
                .map(|_| MergeVisibilityState::Visible)
                .unwrap_or(MergeVisibilityState::NotVisible),
            embedded_surface_state: embedded_target_visibility_state(record, target_record),
            lifecycle: resolved_target.as_ref().map(|metadata| metadata.lifecycle),
            created_at_version: resolved_target
                .as_ref()
                .map(|metadata| metadata.created_at_version),
            retired_at_version: resolved_target
                .as_ref()
                .and_then(|metadata| metadata.retired_at_version),
        };
    }
    match record.record_kind {
        VisibleMergeRecordKind::Entity => embedded_visibility_evidence(
            record.record_ref.clone(),
            MergeVisibilityEvidenceKind::TargetEmbeddedSurface,
            record.target_entity.as_ref().map(|entity| entity.lifecycle),
            record
                .target_entity
                .as_ref()
                .map(|entity| entity.created_at_version),
            record
                .target_entity
                .as_ref()
                .and_then(|entity| entity.retired_at_version),
        ),
        VisibleMergeRecordKind::Relation => embedded_visibility_evidence(
            record.record_ref.clone(),
            MergeVisibilityEvidenceKind::TargetEmbeddedSurface,
            record
                .target_relation
                .as_ref()
                .map(|relation| relation.lifecycle),
            record
                .target_relation
                .as_ref()
                .map(|relation| relation.created_at_version),
            record
                .target_relation
                .as_ref()
                .and_then(|relation| relation.retired_at_version),
        ),
    }
}

pub(super) fn base_record_visibility_evidence(
    record: &VisibleMergeRecord,
    base_version_id: VersionId,
    base_view: &RelationalReadView,
) -> MergeVisibilityEvidence {
    if let Some(base_record) = view_record_visibility_metadata(base_view, &record.record_ref) {
        return MergeVisibilityEvidence {
            observed_record: record.record_ref.clone(),
            kind: MergeVisibilityEvidenceKind::BaseResolvedViewLookup,
            state: MergeVisibilityState::Visible,
            embedded_surface_state: None,
            lifecycle: Some(base_record.lifecycle),
            created_at_version: Some(base_record.created_at_version),
            retired_at_version: base_record.retired_at_version,
        };
    }
    match record.record_kind {
        VisibleMergeRecordKind::Entity => {
            if let Some(entity) = record
                .source_entity
                .as_ref()
                .or(record.target_entity.as_ref())
            {
                return historical_window_visibility_evidence(
                    record.record_ref.clone(),
                    entity.lifecycle,
                    entity.created_at_version,
                    entity.retired_at_version,
                    base_version_id,
                );
            }
            last_resort_base_view_visibility_evidence(
                record.record_ref.clone(),
                base_view,
                &record.record_ref,
            )
        }
        VisibleMergeRecordKind::Relation => {
            if let Some(relation) = record
                .source_relation
                .as_ref()
                .or(record.target_relation.as_ref())
            {
                return historical_window_visibility_evidence(
                    record.record_ref.clone(),
                    relation.lifecycle,
                    relation.created_at_version,
                    relation.retired_at_version,
                    base_version_id,
                );
            }
            last_resort_base_view_visibility_evidence(
                record.record_ref.clone(),
                base_view,
                &record.record_ref,
            )
        }
    }
}

pub(super) fn visibility_evidence_is_visible(evidence: &MergeVisibilityEvidence) -> bool {
    evidence.state == MergeVisibilityState::Visible
}

fn embedded_visibility_evidence(
    observed_record: RecordRef,
    kind: MergeVisibilityEvidenceKind,
    lifecycle: Option<RecordLifecycleState>,
    created_at_version: Option<VersionId>,
    retired_at_version: Option<VersionId>,
) -> MergeVisibilityEvidence {
    let state = lifecycle
        .filter(|lifecycle| is_visible_lifecycle_value(*lifecycle))
        .map(|_| MergeVisibilityState::Visible)
        .unwrap_or(MergeVisibilityState::NotVisible);
    MergeVisibilityEvidence {
        observed_record,
        kind,
        state,
        embedded_surface_state: None,
        lifecycle,
        created_at_version,
        retired_at_version,
    }
}

fn historical_window_visibility_evidence(
    observed_record: RecordRef,
    lifecycle: RecordLifecycleState,
    created_at_version: VersionId,
    retired_at_version: Option<VersionId>,
    base_version_id: VersionId,
) -> MergeVisibilityEvidence {
    let state = if record_existed_at_base(created_at_version, retired_at_version, base_version_id) {
        MergeVisibilityState::Visible
    } else {
        MergeVisibilityState::NotVisible
    };
    MergeVisibilityEvidence {
        observed_record,
        kind: MergeVisibilityEvidenceKind::BaseHistoricalWindow,
        state,
        embedded_surface_state: None,
        lifecycle: Some(lifecycle),
        created_at_version: Some(created_at_version),
        retired_at_version,
    }
}

fn last_resort_base_view_visibility_evidence(
    observed_record: RecordRef,
    view: &RelationalReadView,
    record_ref: &RecordRef,
) -> MergeVisibilityEvidence {
    let state = if view_record_visibility_metadata(view, record_ref).is_some() {
        MergeVisibilityState::Visible
    } else {
        MergeVisibilityState::NotVisible
    };
    MergeVisibilityEvidence {
        observed_record,
        kind: MergeVisibilityEvidenceKind::BaseLastResortViewLookup,
        state,
        embedded_surface_state: None,
        lifecycle: None,
        created_at_version: None,
        retired_at_version: None,
    }
}

#[derive(Debug, Clone, Copy)]
struct ViewRecordVisibilityMetadata {
    lifecycle: RecordLifecycleState,
    created_at_version: VersionId,
    retired_at_version: Option<VersionId>,
}

fn view_record_visibility_metadata(
    view: &RelationalReadView,
    record_ref: &RecordRef,
) -> Option<ViewRecordVisibilityMetadata> {
    match record_ref {
        RecordRef::Entity(entity_id) => {
            view.get_entity(*entity_id)
                .map(|entity| ViewRecordVisibilityMetadata {
                    lifecycle: entity.lifecycle,
                    created_at_version: entity.created_at_version,
                    retired_at_version: entity.retired_at_version,
                })
        }
        RecordRef::Relation(relation_id) => {
            view.get_relation(*relation_id)
                .map(|relation| ViewRecordVisibilityMetadata {
                    lifecycle: relation.lifecycle,
                    created_at_version: relation.created_at_version,
                    retired_at_version: relation.retired_at_version,
                })
        }
    }
}

fn embedded_target_visibility_state(
    record: &VisibleMergeRecord,
    candidate_target_record: &RecordRef,
) -> Option<MergeVisibilityState> {
    if candidate_target_record != &record.record_ref {
        return None;
    }
    match record.record_kind {
        VisibleMergeRecordKind::Entity => record
            .target_entity
            .as_ref()
            .map(|entity| visibility_state_from_lifecycle(entity.lifecycle)),
        VisibleMergeRecordKind::Relation => record
            .target_relation
            .as_ref()
            .map(|relation| visibility_state_from_lifecycle(relation.lifecycle)),
    }
}

fn visibility_state_from_lifecycle(lifecycle: RecordLifecycleState) -> MergeVisibilityState {
    if is_visible_lifecycle_value(lifecycle) {
        MergeVisibilityState::Visible
    } else {
        MergeVisibilityState::NotVisible
    }
}

fn record_existed_at_base(
    created_at_version: VersionId,
    retired_at_version: Option<VersionId>,
    base_version_id: VersionId,
) -> bool {
    created_at_version <= base_version_id
        && retired_at_version
            .map(|retired_at| retired_at > base_version_id)
            .unwrap_or(true)
}

fn is_visible_lifecycle_value(lifecycle: RecordLifecycleState) -> bool {
    !matches!(
        lifecycle,
        RecordLifecycleState::DeletedRetained
            | RecordLifecycleState::RetainedDanglingForAudit
            | RecordLifecycleState::Reclaimable
            | RecordLifecycleState::Reusable
    )
}
