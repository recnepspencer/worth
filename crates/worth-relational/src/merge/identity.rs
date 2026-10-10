use std::sync::Arc;

use crate::merge::aspect_plan_lookup::lowered_plan_for_record;
use crate::merge::data::{
    HistoryScopedMergePlan, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
    IdentityMatchCandidate, IdentityMatchClass, IdentityResolutionReason, IdentityScopedMergePlan,
    MergeAncestrySummary, MergePlanningError, MergeRecordIdentity,
    NormalizedRelationalMergeRequest, VisibleMergeRecord, VisibleMergeRecordKind,
};
use crate::merge::identity_records::{
    effective_identity_declarations, identity_summary, source_visible_records,
    validate_schema_declared_correspondences,
};
use crate::merge::identity_target_index::{
    declared_key_set_match, DeclaredKeySetMatch, TargetIdentityIndex,
};
use crate::merge::planning::branch_delta_summary;
use crate::merge::MergeAccess;
use crate::transactions::data::RecordRef;

impl<'runtime> MergeAccess<'runtime> {
    pub(crate) fn plan_identity_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: NormalizedRelationalMergeRequest,
    ) -> Result<IdentityScopedMergePlan, MergePlanningError> {
        let history_plan = self.plan_history_scope(request)?;
        Ok(self.discover_identity_scope(execution, history_plan))
    }

    fn discover_identity_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history_plan: HistoryScopedMergePlan,
    ) -> IdentityScopedMergePlan {
        let target_view = self
            .runtime
            .read_truth()
            .read_version(execution, history_plan.basis.target_head.version_id);
        let source_view = self
            .runtime
            .read_truth()
            .read_version(execution, history_plan.basis.source_head.version_id);

        let ancestry = MergeAncestrySummary {
            merge_base_rule: history_plan.basis.merge_base.rule,
            merge_base_commit_id: history_plan.basis.merge_base.commit.commit_id,
            supporting_left_ancestor_count: history_plan
                .basis
                .merge_base
                .supporting_left_ancestors
                .len(),
            supporting_right_ancestor_count: history_plan
                .basis
                .merge_base
                .supporting_right_ancestors
                .len(),
            target: branch_delta_summary(
                &history_plan.basis.target_head,
                &history_plan.target_delta,
            ),
            source: branch_delta_summary(
                &history_plan.basis.source_head,
                &history_plan.source_delta,
            ),
        };

        let source_records = source_visible_records(
            &source_view,
            &target_view,
            history_plan.source_delta.touched_records.as_ref(),
        );
        let effective_identity_declarations =
            effective_identity_declarations(self.runtime, &source_records);
        let target_index = TargetIdentityIndex::new(
            self.runtime,
            &target_view,
            source_records.as_slice(),
            &effective_identity_declarations,
        );
        let candidates = source_records
            .iter()
            .map(|record| {
                discover_identity_candidate(
                    self.runtime,
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

        IdentityScopedMergePlan {
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
        }
    }
}

fn discover_identity_candidate(
    runtime: &crate::runtime::RelationalRuntime,
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
                    identity_declaration_applies_to_record(runtime, source, declaration, scope)
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
                match declared_key_set_match(runtime, source, keys.as_ref(), target_index) {
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
    runtime: &crate::runtime::RelationalRuntime,
    source: &VisibleMergeRecord,
    declaration: &IdentityBasisDeclaration,
    kind_scope: &IdentityBasisScope,
) -> bool {
    match &declaration.scope {
        IdentityBasisScope::EntityKind(_) | IdentityBasisScope::RelationKind(_) => {
            &declaration.scope == kind_scope
        }
        IdentityBasisScope::AspectKey(aspect_key) => lowered_plan_for_record(runtime, source)
            .map(|plan| {
                plan.executable_bindings
                    .iter()
                    .any(|binding| binding.aspect_key() == aspect_key)
            })
            .unwrap_or(false),
    }
}
