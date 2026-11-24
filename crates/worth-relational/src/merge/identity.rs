use std::sync::Arc;

use crate::merge::aspect_plan_lookup::lowered_plan_for_record;
use crate::merge::branch_delta::branch_delta_summary;
use crate::merge::data::{
    HistoryScopedMergePlan, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
    IdentityMatchCandidate, IdentityMatchClass, IdentityResolutionReason, IdentityScopedMergePlan,
    MergeAncestrySummary, MergeRecordIdentity, VisibleMergeRecord, VisibleMergeRecordKind,
};
use crate::merge::identity_records::{
    effective_identity_declarations, identity_summary, source_visible_records,
    validate_schema_declared_correspondences,
};
use crate::merge::identity_target_index::{
    declared_key_set_match, DeclaredKeySetMatch, TargetIdentityIndex,
};
use crate::transactions::data::primitives::RecordRef;

use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};
use crate::visibility::snapshot_read_view::RelationalReadView;

pub(super) fn discover_identity_scope(
    registry: &RelationalSchemaRegistry,
    aspect_plans: &AspectContractPlanCatalog,
    target_view: &RelationalReadView,
    source_view: &RelationalReadView,
    history_plan: HistoryScopedMergePlan,
) -> (IdentityScopedMergePlan, usize, usize) {
    let ancestry = MergeAncestrySummary {
        merge_base_rule: history_plan.basis.merge_base().rule(),
        merge_base_commit_id: history_plan.basis.merge_base().commit().commit_id,
        supporting_left_ancestor_count: history_plan
            .basis
            .merge_base()
            .supporting_left_ancestors()
            .len(),
        supporting_right_ancestor_count: history_plan
            .basis
            .merge_base()
            .supporting_right_ancestors()
            .len(),
        target: branch_delta_summary(history_plan.basis.target_head(), &history_plan.target_delta),
        source: branch_delta_summary(history_plan.basis.source_head(), &history_plan.source_delta),
    };

    let source_records = source_visible_records(
        source_view,
        target_view,
        history_plan.source_delta.touched_records.as_ref(),
    );
    let effective_identity_declarations =
        effective_identity_declarations(registry, &source_records);
    let (target_index, indexed_rows) = TargetIdentityIndex::new(
        aspect_plans,
        target_view,
        source_records.as_slice(),
        &effective_identity_declarations,
    );
    let candidates = source_records
        .iter()
        .map(|record| {
            discover_identity_candidate(
                aspect_plans,
                record,
                &effective_identity_declarations,
                &target_index,
            )
        })
        .collect::<Vec<_>>();
    let validated_schema_correspondences =
        validate_schema_declared_correspondences(candidates.as_slice());
    let identity_summary = identity_summary(
        Arc::from(effective_identity_declarations.clone()),
        Arc::from(candidates.clone()),
        Arc::from(validated_schema_correspondences.clone()),
    );

    let plan = IdentityScopedMergePlan {
        request: history_plan.request,
        basis: history_plan.basis,
        ancestry,
        target_delta: history_plan.target_delta,
        source_delta: history_plan.source_delta,
        effective_identity_declarations: Arc::from(effective_identity_declarations),
        source_records: Arc::from(source_records),
        candidates: Arc::from(candidates),
        validated_schema_correspondences: Arc::from(validated_schema_correspondences),
        identity_summary,
    };
    (
        plan,
        target_view.entities().len() + target_view.relations().len(),
        indexed_rows,
    )
}
fn discover_identity_candidate(
    aspect_plans: &AspectContractPlanCatalog,
    source: &VisibleMergeRecord,
    effective_identity_declarations: &[IdentityBasisDeclaration],
    target_index: &TargetIdentityIndex,
) -> IdentityMatchCandidate {
    let kind_scope = source.kind_id.map(|kind_id| match source.record_kind {
        VisibleMergeRecordKind::Entity => IdentityBasisScope::EntityKind(kind_id),
        VisibleMergeRecordKind::Relation => IdentityBasisScope::RelationKind(kind_id),
    });
    let declared_bases = kind_scope
        .as_ref()
        .map(|scope| {
            effective_identity_declarations
                .iter()
                .filter(|declaration| {
                    identity_declaration_applies_to_record(aspect_plans, source, declaration, scope)
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            kind_scope
                .clone()
                .map(|scope| {
                    vec![IdentityBasisDeclaration {
                        scope,
                        basis: IdentityBasisKind::StorageIdentity,
                    }]
                })
                .unwrap_or_default()
        });
    let source_identity = source_identity_for_basis(
        source,
        declared_bases.first().map(|declaration| &declaration.basis),
    );

    for declaration in &declared_bases {
        match &declaration.basis {
            IdentityBasisKind::StorageIdentity => {
                if let Some(target_record) = target_index.storage_match(&source.record_ref) {
                    return IdentityMatchCandidate {
                        scope: Some(declaration.scope.clone()),
                        source_record: source.record_ref.clone(),
                        target_record: Some(target_record.clone()),
                        source: source_identity_for_basis(source, Some(&declaration.basis)),
                        target: Some(MergeRecordIdentity::StorageRecord(target_record)),
                        match_class: IdentityMatchClass::Exact,
                        reason: IdentityResolutionReason::ExactStorageIdentity,
                        basis: declaration.basis.clone(),
                    };
                }
            }
            IdentityBasisKind::LineageIdentity => {
                if let Some(lineage_id) = source.source_lineage_id {
                    match target_index.entities_by_lineage.get(&lineage_id) {
                        Some(matches) if matches.len() == 1 => {
                            let target_record = RecordRef::Entity(matches[0]);
                            return IdentityMatchCandidate {
                                scope: Some(declaration.scope.clone()),
                                source_record: source.record_ref.clone(),
                                target_record: Some(target_record),
                                source: MergeRecordIdentity::Lineage(lineage_id),
                                target: Some(MergeRecordIdentity::Lineage(lineage_id)),
                                match_class: IdentityMatchClass::Exact,
                                reason: IdentityResolutionReason::ExactLineageIdentity,
                                basis: declaration.basis.clone(),
                            };
                        }
                        Some(matches) if matches.len() > 1 => {
                            return IdentityMatchCandidate {
                                scope: Some(declaration.scope.clone()),
                                source_record: source.record_ref.clone(),
                                target_record: None,
                                source: MergeRecordIdentity::Lineage(lineage_id),
                                target: None,
                                match_class: IdentityMatchClass::Ambiguous,
                                reason: IdentityResolutionReason::DeclaredBasisAmbiguousVisibleTargetMatch,
                                basis: declaration.basis.clone(),
                            };
                        }
                        _ => {}
                    }
                } else {
                    return IdentityMatchCandidate {
                        scope: Some(declaration.scope.clone()),
                        source_record: source.record_ref.clone(),
                        target_record: None,
                        source: source_identity.clone(),
                        target: None,
                        match_class: IdentityMatchClass::MissingTarget,
                        reason: IdentityResolutionReason::DeclaredBasisUnavailableOnSource,
                        basis: declaration.basis.clone(),
                    };
                }
            }
            IdentityBasisKind::DeclaredKeySet(keys) => {
                match declared_key_set_match(aspect_plans, source, keys.as_ref(), target_index) {
                    DeclaredKeySetMatch::ExactTarget(target_record) => {
                        return IdentityMatchCandidate {
                            scope: Some(declaration.scope.clone()),
                            source_record: source.record_ref.clone(),
                            target_record: Some(target_record.clone()),
                            source: MergeRecordIdentity::StorageRecord(source.record_ref.clone()),
                            target: Some(MergeRecordIdentity::StorageRecord(target_record)),
                            match_class: IdentityMatchClass::Reconciliable,
                            reason: IdentityResolutionReason::SchemaDeclaredCorrespondence,
                            basis: declaration.basis.clone(),
                        };
                    }
                    DeclaredKeySetMatch::MissingSourceEvidence => {
                        return IdentityMatchCandidate {
                            scope: Some(declaration.scope.clone()),
                            source_record: source.record_ref.clone(),
                            target_record: None,
                            source: source_identity.clone(),
                            target: None,
                            match_class: IdentityMatchClass::MissingTarget,
                            reason: IdentityResolutionReason::DeclaredBasisUnavailableOnSource,
                            basis: declaration.basis.clone(),
                        };
                    }
                    DeclaredKeySetMatch::AmbiguousTarget => {
                        return IdentityMatchCandidate {
                            scope: Some(declaration.scope.clone()),
                            source_record: source.record_ref.clone(),
                            target_record: None,
                            source: source_identity.clone(),
                            target: None,
                            match_class: IdentityMatchClass::Ambiguous,
                            reason:
                                IdentityResolutionReason::DeclaredBasisAmbiguousVisibleTargetMatch,
                            basis: declaration.basis.clone(),
                        };
                    }
                    DeclaredKeySetMatch::NoTargetMatch => {}
                }
            }
            _ => {}
        }
    }

    let unresolved_identity_basis = declared_bases
        .first()
        .map(|declaration| declaration.basis.clone())
        .unwrap_or(IdentityBasisKind::StorageIdentity);
    IdentityMatchCandidate {
        scope: kind_scope,
        source_record: source.record_ref.clone(),
        target_record: None,
        source: source_identity,
        target: None,
        match_class: IdentityMatchClass::MissingTarget,
        reason: IdentityResolutionReason::DeclaredBasisNoVisibleTargetMatch,
        basis: unresolved_identity_basis,
    }
}

fn source_identity_for_basis(
    source: &VisibleMergeRecord,
    basis: Option<&IdentityBasisKind>,
) -> MergeRecordIdentity {
    match basis {
        Some(IdentityBasisKind::LineageIdentity) => source
            .source_lineage_id
            .map(MergeRecordIdentity::Lineage)
            .unwrap_or_else(|| MergeRecordIdentity::StorageRecord(source.record_ref.clone())),
        _ => MergeRecordIdentity::StorageRecord(source.record_ref.clone()),
    }
}

fn identity_declaration_applies_to_record(
    aspect_plans: &AspectContractPlanCatalog,
    source: &VisibleMergeRecord,
    declaration: &IdentityBasisDeclaration,
    kind_scope: &IdentityBasisScope,
) -> bool {
    match &declaration.scope {
        IdentityBasisScope::EntityKind(_) | IdentityBasisScope::RelationKind(_) => {
            &declaration.scope == kind_scope
        }
        IdentityBasisScope::AspectKey(aspect_key) => lowered_plan_for_record(aspect_plans, source)
            .map(|plan| {
                plan.executable_bindings
                    .iter()
                    .any(|binding| binding.aspect_key() == aspect_key)
            })
            .unwrap_or(false),
    }
}
