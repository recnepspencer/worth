use crate::contracts::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::merge::conflicts::ancestor_record_basis::{
    AncestorEntityRecordBasis, AncestorRecordBasisContext, AncestorRelationRecordBasis,
};
use crate::merge::data::{
    DeletionMergeClass, IdentityMatchClass, MergeConflictClass, VisibleMergeRecord,
    VisibleMergeRecordKind,
};

pub(super) fn classify_record_state(
    record: &VisibleMergeRecord,
    base_record_visible: bool,
    source_record_visible: bool,
    target_record_visible: bool,
    match_class: IdentityMatchClass,
    ancestor_basis: &AncestorRecordBasisContext<'_>,
    target_record: Option<&RecordRef>,
) -> MergeConflictClass {
    match (source_record_visible, target_record_visible) {
        (false, true) => MergeConflictClass::Deletion(classify_source_deleted(
            record,
            target_record,
            ancestor_basis,
        )),
        (true, false) => {
            if base_record_visible {
                MergeConflictClass::Deletion(classify_target_deleted(
                    record,
                    target_record,
                    ancestor_basis,
                ))
            } else {
                MergeConflictClass::SourceOnlyAddition
            }
        }
        (false, false) => MergeConflictClass::Deletion(DeletionMergeClass::DeletedOnBothSides),
        (true, true) => match match_class {
            IdentityMatchClass::Exact => classify_visible_exact_state(record),
            IdentityMatchClass::Reconciliable => MergeConflictClass::SchemaDeclaredCorrespondence,
            IdentityMatchClass::Ambiguous | IdentityMatchClass::MissingTarget => {
                MergeConflictClass::DivergentVisibleState
            }
        },
    }
}

fn classify_source_deleted(
    record: &VisibleMergeRecord,
    target_record: Option<&RecordRef>,
    ancestor_basis: &AncestorRecordBasisContext<'_>,
) -> DeletionMergeClass {
    match record.record_kind {
        VisibleMergeRecordKind::Entity => {
            match (
                ancestor_basis.entity_basis(record, target_record),
                record.target_entity.as_ref(),
            ) {
                (Some(base), Some(target)) if !entity_matches_ancestor(target, &base) => {
                    DeletionMergeClass::DeletedVsModified
                }
                _ => DeletionMergeClass::SourceDeletedTargetLive,
            }
        }
        VisibleMergeRecordKind::Relation => {
            match (
                ancestor_basis.relation_basis(record, target_record),
                record.target_relation.as_ref(),
            ) {
                (Some(base), Some(target)) if !relation_endpoints_match_ancestor(target, &base) => {
                    DeletionMergeClass::DeletedVsRewired
                }
                (Some(base), Some(target)) if !relation_matches_ancestor(target, &base) => {
                    DeletionMergeClass::DeletedVsModified
                }
                _ => DeletionMergeClass::SourceDeletedTargetLive,
            }
        }
    }
}

fn classify_target_deleted(
    record: &VisibleMergeRecord,
    target_record: Option<&RecordRef>,
    ancestor_basis: &AncestorRecordBasisContext<'_>,
) -> DeletionMergeClass {
    match record.record_kind {
        VisibleMergeRecordKind::Entity => {
            match (
                ancestor_basis.entity_basis(record, target_record),
                record.source_entity.as_ref(),
            ) {
                (Some(base), Some(source)) if !entity_matches_ancestor(source, &base) => {
                    DeletionMergeClass::DeletedVsModified
                }
                _ => DeletionMergeClass::SourceLiveTargetDeleted,
            }
        }
        VisibleMergeRecordKind::Relation => {
            match (
                ancestor_basis.relation_basis(record, target_record),
                record.source_relation.as_ref(),
            ) {
                (Some(base), Some(source)) if !relation_endpoints_match_ancestor(source, &base) => {
                    DeletionMergeClass::DeletedVsRewired
                }
                (Some(base), Some(source)) if !relation_matches_ancestor(source, &base) => {
                    DeletionMergeClass::DeletedVsModified
                }
                _ => DeletionMergeClass::SourceLiveTargetDeleted,
            }
        }
    }
}

fn classify_visible_exact_state(record: &VisibleMergeRecord) -> MergeConflictClass {
    match record.record_kind {
        VisibleMergeRecordKind::Entity => {
            match (record.source_entity.as_ref(), record.target_entity.as_ref()) {
                (Some(source), Some(target)) => {
                    if entity_state_equal(source, target) {
                        MergeConflictClass::ExactSharedTruth
                    } else {
                        MergeConflictClass::DivergentVisibleState
                    }
                }
                _ => MergeConflictClass::DivergentVisibleState,
            }
        }
        VisibleMergeRecordKind::Relation => match (
            record.source_relation.as_ref(),
            record.target_relation.as_ref(),
        ) {
            (Some(source), Some(target)) => {
                if relation_endpoints_equal(source, target) {
                    if relation_state_equal(source, target) {
                        MergeConflictClass::ExactSharedTruth
                    } else {
                        MergeConflictClass::DivergentVisibleState
                    }
                } else {
                    MergeConflictClass::RelationEndpointDivergence
                }
            }
            _ => MergeConflictClass::DivergentVisibleState,
        },
    }
}

fn entity_state_equal(source: &EntityReadRecord, target: &EntityReadRecord) -> bool {
    source.lifecycle == target.lifecycle
        && source.authoritative_aspect_state == target.authoritative_aspect_state
}

fn relation_state_equal(source: &RelationReadRecord, target: &RelationReadRecord) -> bool {
    source.lifecycle == target.lifecycle
        && source.authoritative_aspect_state == target.authoritative_aspect_state
}

fn relation_endpoints_equal(source: &RelationReadRecord, target: &RelationReadRecord) -> bool {
    source.source == target.source && source.target == target.target
}

pub(super) fn entity_matches_ancestor(
    record: &EntityReadRecord,
    ancestor: &AncestorEntityRecordBasis<'_>,
) -> bool {
    record.lifecycle == ancestor.lifecycle()
        && record.authoritative_aspect_state.as_ref() == ancestor.authoritative_state()
}

pub(super) fn relation_matches_ancestor(
    record: &RelationReadRecord,
    ancestor: &AncestorRelationRecordBasis<'_>,
) -> bool {
    record.lifecycle == ancestor.lifecycle()
        && record.authoritative_aspect_state.as_ref() == ancestor.authoritative_state()
}

pub(super) fn relation_endpoints_match_ancestor(
    record: &RelationReadRecord,
    ancestor: &AncestorRelationRecordBasis<'_>,
) -> bool {
    record.source == ancestor.source_endpoint() && record.target == ancestor.target_endpoint()
}
