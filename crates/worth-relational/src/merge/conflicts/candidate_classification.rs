use std::collections::BTreeMap;
use std::sync::Arc;

use crate::identity::data::VersionId;
use crate::merge::conflicts::ancestor_record_basis::AncestorRecordBasisContext;
use crate::merge::conflicts::aspect_evidence::aspect_conflict_evidence;
use crate::merge::conflicts::record_state_classification::classify_record_state;
use crate::merge::conflicts::relation_evidence::relation_conflict_evidence;
use crate::merge::conflicts::strategy_evidence::{
    overlapping_strategy_intent_diverges, strategy_conflict_evidence,
};
use crate::merge::conflicts::target_record_resolution::visible_target_record;
use crate::merge::conflicts::visibility_evidence::{
    base_record_visibility_evidence, source_record_visibility_evidence,
    target_record_visibility_evidence, visibility_evidence_is_visible,
};
use crate::merge::data::{
    IdentityMatchClass, IdentityResolutionReason, MergeConflictClass, MergeConflictClassification,
    RelationConflictPropagation, VisibleMergeRecord,
};
use crate::storage::data::RelationalReadView;
use crate::transactions::data::RecordRef;

pub(super) fn classify_candidate(
    runtime: &crate::runtime::RelationalRuntime,
    record: &VisibleMergeRecord,
    candidate_target_record: Option<RecordRef>,
    has_validated_schema_correspondence: bool,
    base_version_id: VersionId,
    base_view: &RelationalReadView,
    ancestor_basis: &AncestorRecordBasisContext<'_>,
    target_view: &RelationalReadView,
    source_touched_by_record: &BTreeMap<RecordRef, &crate::merge::data::BranchTouchedRecordDelta>,
    target_touched_by_record: &BTreeMap<RecordRef, &crate::merge::data::BranchTouchedRecordDelta>,
    match_class: IdentityMatchClass,
    reason: IdentityResolutionReason,
) -> MergeConflictClassification {
    let base_visibility_evidence =
        base_record_visibility_evidence(record, base_version_id, base_view);
    let source_visibility_evidence = source_record_visibility_evidence(record);
    let target_visibility_evidence =
        target_record_visibility_evidence(record, candidate_target_record.as_ref(), target_view);
    let base_record_visible = visibility_evidence_is_visible(&base_visibility_evidence);
    let source_record_visible = visibility_evidence_is_visible(&source_visibility_evidence);
    let target_record_visible = visibility_evidence_is_visible(&target_visibility_evidence);
    let target_record = candidate_target_record
        .as_ref()
        .and_then(|target_record| visible_target_record(target_view, target_record));
    let aspect_evidence =
        aspect_conflict_evidence(runtime, record, target_record.as_ref().or(Some(record)));
    let normalized_match_class = normalize_match_class_for_classification(
        match_class,
        reason.clone(),
        has_validated_schema_correspondence,
    );
    let strategy_evidence = strategy_conflict_evidence(
        source_touched_by_record.get(&record.record_ref).copied(),
        candidate_target_record
            .as_ref()
            .or(Some(&record.record_ref))
            .and_then(|target_record| target_touched_by_record.get(target_record).copied()),
    );
    let relation_evidence =
        relation_conflict_evidence(record, target_record.as_ref(), ancestor_basis);
    let class = if has_validated_schema_correspondence {
        match relation_evidence.as_ref() {
            Some(evidence)
                if evidence.propagation != RelationConflictPropagation::RelationLocalOnly =>
            {
                MergeConflictClass::RelationEndpointDivergence
            }
            _ => MergeConflictClass::SchemaDeclaredCorrespondence,
        }
    } else {
        classify_record_state(
            record,
            base_record_visible,
            source_record_visible,
            target_record_visible,
            normalized_match_class,
            ancestor_basis,
            candidate_target_record.as_ref(),
        )
    };
    let class = if strategy_evidence
        .as_ref()
        .is_some_and(|evidence| overlapping_strategy_intent_diverges(evidence, &aspect_evidence))
        && class == MergeConflictClass::DivergentVisibleState
    {
        MergeConflictClass::StrategyIntentConflict
    } else {
        class
    };

    MergeConflictClassification {
        record: record.record_ref.clone(),
        class,
        identity_reason: reason,
        validated_schema_correspondence: has_validated_schema_correspondence,
        aspect_evidence: Arc::from(aspect_evidence),
        strategy_evidence,
        relation_evidence,
        target_record: candidate_target_record,
        base_record_visible,
        source_record_visible,
        target_record_visible,
        base_visibility_evidence,
        source_visibility_evidence,
        target_visibility_evidence,
    }
}

fn normalize_match_class_for_classification(
    match_class: IdentityMatchClass,
    reason: IdentityResolutionReason,
    has_validated_schema_correspondence: bool,
) -> IdentityMatchClass {
    if reason == IdentityResolutionReason::SchemaDeclaredCorrespondence
        && !has_validated_schema_correspondence
    {
        IdentityMatchClass::Ambiguous
    } else {
        match_class
    }
}
