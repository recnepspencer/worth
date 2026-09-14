use worth_foundational::facade::{AspectBinding, AuthoritativeAspectChangeKind as Change};

use crate::input::envelope::{
    BridgeAspectChangePrecision, BridgeCommittedPatchEnvelope, BridgeCommittedRecordChangeKind,
};

use super::{
    BridgeCorrespondenceDeliveryDenial, BridgeCorrespondenceDenialKind,
    BridgeDeliveredCorrespondenceChange, BridgeSemanticDependencyCandidate,
    CorrespondenceDeliveryCounters, InstalledCorrespondenceTarget,
};

pub(crate) struct BridgeMatchedCorrespondenceChanges {
    pub(crate) counters: CorrespondenceDeliveryCounters,
    pub(crate) changes: Vec<BridgeDeliveredCorrespondenceChange>,
}

pub(crate) fn match_envelope(
    dependency: &BridgeSemanticDependencyCandidate,
    targets: &[InstalledCorrespondenceTarget],
    envelope: &BridgeCommittedPatchEnvelope,
    mut counters: CorrespondenceDeliveryCounters,
) -> Result<BridgeMatchedCorrespondenceChanges, BridgeCorrespondenceDeliveryDenial> {
    let mut changes = Vec::new();
    let source_partition = envelope
        .producer_metadata()
        .authoritative_source()
        .and_then(|source| source.partition_role());
    for item in envelope.patch_body().canonical_items() {
        counters.correspondence_lookups += 1;
        let Some(change) = item.semantic_change() else {
            continue;
        };
        counters.semantic_match_checks += 1;
        if !semantic_change_matches_basis(
            SemanticMatchBasis {
                dependency,
                record: item.relational_record_identity_parts(),
                source_partition,
            },
            change,
            &mut counters,
        ) {
            continue;
        }
        if change.precision() == BridgeAspectChangePrecision::DeclaredWidening {
            let source_widening_admitted = targets.iter().all(|target| {
                counters.source_widening_target_checks += 1;
                target.admitted_source_widening == change.widening_cause()
            });
            if !source_widening_admitted {
                counters.failed_deliveries += 1;
                return Err(BridgeCorrespondenceDeliveryDenial::new(
                    BridgeCorrespondenceDenialKind::MappingSemanticMismatch,
                    counters,
                ));
            }
        }
        counters.truth_targets_admitted += 1;
        changes.push(BridgeDeliveredCorrespondenceChange::semantic_aspect(
            item.entity_identity().into(),
            item.relational_record_identity_parts(),
            change.clone(),
        ));
    }

    if counters.truth_targets_admitted == 0 {
        changes.extend(match_structural_changes(
            dependency,
            envelope,
            source_partition,
            &mut counters,
        ));
    }
    Ok(BridgeMatchedCorrespondenceChanges { counters, changes })
}

struct SemanticMatchBasis<'a> {
    dependency: &'a BridgeSemanticDependencyCandidate,
    record: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    source_partition: Option<&'a worth_foundational::facade::TruthPartitionRole>,
}

fn semantic_change_matches_basis(
    basis: SemanticMatchBasis<'_>,
    change: &crate::input::envelope::BridgeSemanticAspectChange,
    counters: &mut CorrespondenceDeliveryCounters,
) -> bool {
    if basis.dependency.contract.key() != change.aspect_key()
        || basis.dependency.contract.identity() != change.aspect_identity()
        || basis.dependency.contract.revision() != change.contract_revision()
    {
        counters.aspect_rejections += 1;
        return false;
    }
    if &basis.dependency.binding != change.binding() {
        counters.binding_rejections += 1;
        return false;
    }
    if !basis.dependency.relevant_changes.iter().any(|candidate| {
        counters.relevant_change_checks += 1;
        change.intersects_relevant_change(*candidate)
    }) {
        counters.change_kind_rejections += 1;
        return false;
    }
    if !locality_matches(
        &basis.dependency.locality,
        basis.dependency.source_record_identity,
        basis.record,
        basis.source_partition,
    ) {
        counters.locality_rejections += 1;
        return false;
    }
    let projection_matches = basis.dependency.projection_mask.is_whole_aspect()
        || matches!(
            change.kind(),
            Change::WholeAspectSet | Change::WholeAspectClear
        )
        || change.effective_field_path().is_none_or(|path| {
            basis
                .dependency
                .projection_mask
                .paths()
                .iter()
                .any(|candidate| {
                    counters.projection_paths_inspected += 1;
                    canonical_paths_overlap(candidate, path)
                })
        });
    if !projection_matches {
        counters.projection_rejections += 1;
    }
    projection_matches
}

fn match_structural_changes(
    dependency: &BridgeSemanticDependencyCandidate,
    envelope: &BridgeCommittedPatchEnvelope,
    source_partition: Option<&worth_foundational::facade::TruthPartitionRole>,
    counters: &mut CorrespondenceDeliveryCounters,
) -> Vec<BridgeDeliveredCorrespondenceChange> {
    let mut changes = Vec::new();
    for change in envelope.patch_body().canonical_record_changes() {
        counters.correspondence_lookups += 1;
        counters.semantic_match_checks += 1;
        if structural_change_matches(dependency, change, source_partition, counters) {
            counters.truth_targets_admitted += 1;
            changes.push(BridgeDeliveredCorrespondenceChange::structural_record(
                change.clone(),
            ));
        }
    }
    changes
}

fn canonical_paths_overlap(
    left: &worth_foundational::facade::CanonicalFieldPath,
    right: &worth_foundational::facade::CanonicalFieldPath,
) -> bool {
    let shared = left.fields().len().min(right.fields().len());
    left.fields()[..shared] == right.fields()[..shared]
}

fn contains_change(
    admitted: &[Change],
    expected: Change,
    counters: &mut CorrespondenceDeliveryCounters,
) -> bool {
    admitted.iter().any(|candidate| {
        counters.relevant_change_checks += 1;
        *candidate == expected
    })
}

fn structural_change_matches(
    dependency: &BridgeSemanticDependencyCandidate,
    change: &crate::input::envelope::BridgeCommittedRecordChange,
    source_partition: Option<&worth_foundational::facade::TruthPartitionRole>,
    counters: &mut CorrespondenceDeliveryCounters,
) -> bool {
    let meaning = structural_change_kind(&dependency.binding, change.kind());
    meaning.is_some_and(|meaning| contains_change(&dependency.relevant_changes, meaning, counters))
        && locality_matches(
            &dependency.locality,
            dependency.source_record_identity,
            Some(change.record_identity()),
            source_partition,
        )
}

pub(crate) fn structural_change_kind(
    binding: &AspectBinding,
    kind: BridgeCommittedRecordChangeKind,
) -> Option<Change> {
    match (binding, kind) {
        (
            AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet,
            BridgeCommittedRecordChangeKind::Created,
        ) => Some(Change::StructuralCreate),
        (
            AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet,
            BridgeCommittedRecordChangeKind::Updated,
        ) => Some(Change::StructuralUpdate),
        (
            AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet,
            BridgeCommittedRecordChangeKind::Deleted,
        ) => Some(Change::StructuralDelete),
        (
            AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet,
            BridgeCommittedRecordChangeKind::RetainedForAudit,
        ) => Some(Change::StructuralRetainForAudit),
        (
            AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet,
            BridgeCommittedRecordChangeKind::MaterializationSuspended,
        ) => Some(Change::StructuralMaterializationSuspended),
        (
            AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet,
            BridgeCommittedRecordChangeKind::Rematerialized,
        ) => Some(Change::StructuralRematerialized),
        (AspectBinding::LifecycleTransition, BridgeCommittedRecordChangeKind::Created) => {
            Some(Change::LifecycleCreate)
        }
        (AspectBinding::LifecycleTransition, BridgeCommittedRecordChangeKind::Deleted) => {
            Some(Change::LifecycleDelete)
        }
        (AspectBinding::LifecycleTransition, BridgeCommittedRecordChangeKind::RetainedForAudit) => {
            Some(Change::LifecycleRetainForAudit)
        }
        _ => None,
    }
}

#[cfg(test)]
mod structural_kind_tests {
    use super::*;

    #[test]
    fn record_changes_retain_binding_aware_semantic_kinds() {
        assert_eq!(
            structural_change_kind(
                &AspectBinding::StructuralRegion,
                BridgeCommittedRecordChangeKind::Created,
            ),
            Some(Change::StructuralCreate)
        );
        assert_eq!(
            structural_change_kind(
                &AspectBinding::LifecycleTransition,
                BridgeCommittedRecordChangeKind::Deleted,
            ),
            Some(Change::LifecycleDelete)
        );
        assert_eq!(
            structural_change_kind(
                &AspectBinding::LifecycleTransition,
                BridgeCommittedRecordChangeKind::Updated,
            ),
            None
        );
    }
}

fn locality_matches(
    locality: &super::BridgeSemanticLocality,
    source_record_identity: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    record: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    source_partition: Option<&worth_foundational::facade::TruthPartitionRole>,
) -> bool {
    match locality {
        super::BridgeSemanticLocality::WholeLogicalGraph => true,
        super::BridgeSemanticLocality::SourceRecord => matches!(
            (source_record_identity, record),
            (Some(expected), Some(actual)) if expected == actual
        ),
        super::BridgeSemanticLocality::ManagedSourceRecord => record.is_some(),
        super::BridgeSemanticLocality::SourcePartition(role) => {
            source_partition == Some(role)
                && source_record_identity.is_none_or(|expected| record == Some(expected))
        }
    }
}

#[cfg(test)]
mod path_overlap_tests {
    use super::canonical_paths_overlap;
    use worth_foundational::facade::{CanonicalFieldPath, FieldKey};

    fn path(fields: &[&str]) -> CanonicalFieldPath {
        CanonicalFieldPath::new(
            fields
                .iter()
                .map(|field| FieldKey::new(*field).unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    #[test]
    fn semantic_delivery_uses_parent_child_overlap_but_rejects_siblings() {
        let parent = path(&["profile"]);
        let child = path(&["profile", "name"]);
        let sibling = path(&["status"]);
        assert!(canonical_paths_overlap(&parent, &child));
        assert!(canonical_paths_overlap(&child, &parent));
        assert!(!canonical_paths_overlap(&child, &sibling));
    }
}
