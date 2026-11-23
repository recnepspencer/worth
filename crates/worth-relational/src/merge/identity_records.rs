use std::collections::BTreeMap;
use std::sync::Arc;

use crate::merge::data::{
    BranchTouchedRecordDelta, IdentityBasisDeclaration, IdentityBasisKind,
    IdentityDiscoverySummary, IdentityMatchCandidate, IdentityMatchClass, IdentityResolutionReason,
    SchemaDeclaredCorrespondenceValidationSummary, ValidatedSchemaDeclaredCorrespondence,
    VisibleMergeRecord, VisibleMergeRecordKind,
};
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::primitives::RecordRef;
use crate::visibility::snapshot_read_view::RelationalReadView;

pub(crate) fn source_visible_records(
    source_view: &RelationalReadView,
    target_view: &RelationalReadView,
    touched_records: &[BranchTouchedRecordDelta],
) -> Vec<VisibleMergeRecord> {
    touched_records
        .iter()
        .map(|delta| visible_record_for_ref(source_view, target_view, delta.target.clone()))
        .collect()
}

pub(crate) fn effective_identity_declarations(
    registry: &RelationalSchemaRegistry,
    source_records: &[VisibleMergeRecord],
) -> Vec<IdentityBasisDeclaration> {
    let mut declarations = Vec::<IdentityBasisDeclaration>::new();
    for record in source_records {
        match record.record_kind {
            VisibleMergeRecordKind::Entity => {
                let Some(kind_id) = record.kind_id else {
                    continue;
                };
                if let Ok(schema_declarations) = registry.entity_identity_declarations(kind_id) {
                    for declaration in schema_declarations {
                        push_unique_declaration(&mut declarations, declaration.clone());
                    }
                }
            }
            VisibleMergeRecordKind::Relation => {
                let Some(kind_id) = record.kind_id else {
                    continue;
                };
                if let Ok(schema_declarations) = registry.relation_identity_declarations(kind_id) {
                    for declaration in schema_declarations {
                        push_unique_declaration(&mut declarations, declaration.clone());
                    }
                }
            }
        }
    }
    declarations
}

pub(crate) fn identity_summary(
    effective_declarations: Arc<[IdentityBasisDeclaration]>,
    candidates: Arc<[IdentityMatchCandidate]>,
    validated_schema_correspondences: Arc<[ValidatedSchemaDeclaredCorrespondence]>,
) -> IdentityDiscoverySummary {
    let schema_declared_candidates = candidates
        .iter()
        .filter(|candidate| {
            candidate.reason == IdentityResolutionReason::SchemaDeclaredCorrespondence
        })
        .collect::<Vec<_>>();
    let schema_uniqueness = schema_declared_candidate_uniqueness(&schema_declared_candidates);
    let candidate_counts = candidate_class_counts(candidates.as_ref());

    IdentityDiscoverySummary {
        effective_declarations,
        candidate_count: candidates.len(),
        exact_match_count: candidate_counts.exact_match_count,
        reconciliable_match_count: candidate_counts.reconciliable_match_count,
        schema_declared_correspondence: SchemaDeclaredCorrespondenceValidationSummary {
            candidate_count: schema_declared_candidates.len(),
            validated_count: validated_schema_correspondences.len(),
            rejected_non_unique_source_count: schema_uniqueness.rejected_non_unique_source_count,
            rejected_non_unique_target_count: schema_uniqueness.rejected_non_unique_target_count,
        },
        ambiguous_match_count: candidate_counts.ambiguous_match_count,
        missing_target_count: candidate_counts.missing_target_count,
        storage_basis_candidate_count: candidate_counts.storage_basis_candidate_count,
        lineage_basis_candidate_count: candidate_counts.lineage_basis_candidate_count,
        structural_basis_candidate_count: candidate_counts.structural_basis_candidate_count,
        custom_basis_candidate_count: candidate_counts.custom_basis_candidate_count,
        candidates,
    }
}

pub(crate) fn validate_schema_declared_correspondences(
    candidates: &[IdentityMatchCandidate],
) -> Vec<ValidatedSchemaDeclaredCorrespondence> {
    let raw = schema_declared_raw_correspondences(candidates);
    let source_counts = correspondence_counts_by_source(&raw);
    let target_counts = correspondence_counts_by_target(&raw);

    raw.into_iter()
        .filter_map(|correspondence| {
            let source_count = source_counts
                .get(&correspondence.source_record)
                .copied()
                .unwrap_or(0);
            let target_count = target_counts
                .get(&correspondence.target_record)
                .copied()
                .unwrap_or(0);
            (source_count == 1 && target_count == 1).then_some(
                ValidatedSchemaDeclaredCorrespondence {
                    candidate_count_for_source: source_count,
                    candidate_count_for_target: target_count,
                    ..correspondence
                },
            )
        })
        .collect()
}

fn visible_record_for_ref(
    source_view: &RelationalReadView,
    target_view: &RelationalReadView,
    record_ref: RecordRef,
) -> VisibleMergeRecord {
    match record_ref {
        RecordRef::Entity(entity_id) => {
            let source = source_view.get_entity(entity_id).cloned();
            let target = target_view.get_entity(entity_id).cloned();
            VisibleMergeRecord {
                record_ref: RecordRef::Entity(entity_id),
                record_kind: VisibleMergeRecordKind::Entity,
                kind_id: source
                    .as_ref()
                    .map(|record| record.kind.kind_id)
                    .or_else(|| target.as_ref().map(|record| record.kind.kind_id)),
                source_kind_id: source.as_ref().map(|record| record.kind.kind_id),
                target_kind_id: target.as_ref().map(|record| record.kind.kind_id),
                lineage_id: source
                    .as_ref()
                    .and_then(|record| record.lineage_id)
                    .or_else(|| target.as_ref().and_then(|record| record.lineage_id)),
                source_lineage_id: source.as_ref().and_then(|record| record.lineage_id),
                target_lineage_id: target.as_ref().and_then(|record| record.lineage_id),
                source_entity: source,
                target_entity: target,
                source_relation: None,
                target_relation: None,
            }
        }
        RecordRef::Relation(relation_id) => {
            let source = source_view.get_relation(relation_id).cloned();
            let target = target_view.get_relation(relation_id).cloned();
            VisibleMergeRecord {
                record_ref: RecordRef::Relation(relation_id),
                record_kind: VisibleMergeRecordKind::Relation,
                kind_id: source
                    .as_ref()
                    .map(|record| record.kind.kind_id)
                    .or_else(|| target.as_ref().map(|record| record.kind.kind_id)),
                source_kind_id: source.as_ref().map(|record| record.kind.kind_id),
                target_kind_id: target.as_ref().map(|record| record.kind.kind_id),
                lineage_id: None,
                source_lineage_id: None,
                target_lineage_id: None,
                source_entity: None,
                target_entity: None,
                source_relation: source,
                target_relation: target,
            }
        }
    }
}

fn push_unique_declaration(
    declarations: &mut Vec<IdentityBasisDeclaration>,
    candidate: IdentityBasisDeclaration,
) {
    if !declarations.iter().any(|existing| existing == &candidate) {
        declarations.push(candidate);
    }
}

#[derive(Default)]
struct CandidateClassCounts {
    exact_match_count: usize,
    reconciliable_match_count: usize,
    ambiguous_match_count: usize,
    missing_target_count: usize,
    storage_basis_candidate_count: usize,
    lineage_basis_candidate_count: usize,
    structural_basis_candidate_count: usize,
    custom_basis_candidate_count: usize,
}

fn candidate_class_counts(candidates: &[IdentityMatchCandidate]) -> CandidateClassCounts {
    let mut counts = CandidateClassCounts::default();
    for candidate in candidates {
        match candidate.match_class {
            IdentityMatchClass::Exact => counts.exact_match_count += 1,
            IdentityMatchClass::Reconciliable => counts.reconciliable_match_count += 1,
            IdentityMatchClass::Ambiguous => counts.ambiguous_match_count += 1,
            IdentityMatchClass::MissingTarget => counts.missing_target_count += 1,
        }
        match &candidate.basis {
            IdentityBasisKind::StorageIdentity => counts.storage_basis_candidate_count += 1,
            IdentityBasisKind::LineageIdentity => counts.lineage_basis_candidate_count += 1,
            IdentityBasisKind::StructuralFingerprint => {
                counts.structural_basis_candidate_count += 1
            }
            IdentityBasisKind::DeclaredKeySet(_) | IdentityBasisKind::Custom(_) => {
                counts.custom_basis_candidate_count += 1
            }
        }
    }
    counts
}

struct SchemaDeclaredCandidateUniqueness {
    rejected_non_unique_source_count: usize,
    rejected_non_unique_target_count: usize,
}

fn schema_declared_candidate_uniqueness(
    schema_declared_candidates: &[&IdentityMatchCandidate],
) -> SchemaDeclaredCandidateUniqueness {
    let mut schema_source_counts = BTreeMap::<RecordRef, usize>::new();
    let mut schema_target_counts = BTreeMap::<RecordRef, usize>::new();
    for candidate in schema_declared_candidates {
        *schema_source_counts
            .entry(candidate.source_record.clone())
            .or_insert(0) += 1;
        if let Some(target_record) = &candidate.target_record {
            *schema_target_counts
                .entry(target_record.clone())
                .or_insert(0) += 1;
        }
    }
    SchemaDeclaredCandidateUniqueness {
        rejected_non_unique_source_count: schema_source_counts
            .values()
            .filter(|count| **count > 1)
            .count(),
        rejected_non_unique_target_count: schema_target_counts
            .values()
            .filter(|count| **count > 1)
            .count(),
    }
}

fn schema_declared_raw_correspondences(
    candidates: &[IdentityMatchCandidate],
) -> Vec<ValidatedSchemaDeclaredCorrespondence> {
    candidates
        .iter()
        .filter_map(|candidate| {
            if candidate.reason != IdentityResolutionReason::SchemaDeclaredCorrespondence {
                return None;
            }
            let scope = candidate.scope.clone()?;
            let target_record = candidate.target_record.clone()?;
            Some(ValidatedSchemaDeclaredCorrespondence {
                scope,
                basis: candidate.basis.clone(),
                source_record: candidate.source_record.clone(),
                target_record,
                candidate_count_for_source: 0,
                candidate_count_for_target: 0,
            })
        })
        .collect()
}

fn correspondence_counts_by_source(
    correspondences: &[ValidatedSchemaDeclaredCorrespondence],
) -> BTreeMap<RecordRef, usize> {
    let mut source_counts = BTreeMap::<RecordRef, usize>::new();
    for correspondence in correspondences {
        *source_counts
            .entry(correspondence.source_record.clone())
            .or_insert(0) += 1;
    }
    source_counts
}

fn correspondence_counts_by_target(
    correspondences: &[ValidatedSchemaDeclaredCorrespondence],
) -> BTreeMap<RecordRef, usize> {
    let mut target_counts = BTreeMap::<RecordRef, usize>::new();
    for correspondence in correspondences {
        *target_counts
            .entry(correspondence.target_record.clone())
            .or_insert(0) += 1;
    }
    target_counts
}
