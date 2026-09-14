use crate::publication::patch::data::{
    PublishedAspectChangePrecision, PublishedAuthoritativeAspectChange,
    PublishedAuthoritativePatchEnvelope, PublishedAuthoritativePatchOperation,
    PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use std::collections::{BTreeMap, HashMap};
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey,
    AuthoritativeAspectChangeKind, CanonicalFieldPath,
};
use worth_runtime_bridge::facade::{
    BridgeAspectChangeWideningCause, BridgeAuthoritativePatchLoweringCounters, BridgeRouteError,
    BridgeRouteErrorKind,
};

use super::RelationalBridgePublicationDenial;

pub(super) fn validate_authoritative_patch_semantics(
    envelope: &PublishedAuthoritativePatchEnvelope,
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
) -> Result<BridgeAuthoritativePatchLoweringCounters, RelationalBridgePublicationDenial> {
    let mut counters = BridgeAuthoritativePatchLoweringCounters::default();
    for record in &envelope.authoritative_record_patches {
        counters.record_patches_inspected += 1;
        counters.authoritative_operations_inspected +=
            record.authoritative_patch.full_grammar_operation_count() as u64;
        if let Err(error) = validate_record(record, admitted_widening, &mut counters) {
            return Err(RelationalBridgePublicationDenial::new(error, counters));
        }
    }
    retain_emitted_target_counters(envelope, admitted_widening, &mut counters);
    Ok(counters)
}

fn validate_record(
    record: &PublishedAuthoritativeRecordPatch,
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
    counters: &mut BridgeAuthoritativePatchLoweringCounters,
) -> Result<(), BridgeRouteError> {
    let expected = expected_changes(record);
    counters.expected_operations_materialized += expected.len() as u64;
    require_exact_semantic_change_count(record, expected.len())?;
    let contains_opaque = inspect_semantic_change_precision(record, counters);
    require_opaque_widening_admission(contains_opaque, admitted_widening)?;
    let mut remaining = expected
        .into_iter()
        .fold(HashMap::new(), |mut counts, change| {
            *counts.entry(change).or_insert(0_usize) += 1;
            counts
        });
    consume_record_semantic_changes(record, &mut remaining, counters)?;
    require_complete_semantic_coverage(&remaining)?;
    if contains_opaque != record.contains_opaque_aspect {
        return Err(denial(
            "opaque aspect posture did not match canonical semantic changes",
        ));
    }
    Ok(())
}

fn require_exact_semantic_change_count(
    record: &PublishedAuthoritativeRecordPatch,
    expected_count: usize,
) -> Result<(), BridgeRouteError> {
    if expected_count == record.semantic_changes.len() {
        return Ok(());
    }
    Err(denial(
        "semantic change count did not match canonical patch operations",
    ))
}

fn inspect_semantic_change_precision(
    record: &PublishedAuthoritativeRecordPatch,
    counters: &mut BridgeAuthoritativePatchLoweringCounters,
) -> bool {
    let mut contains_opaque = false;
    for change in &record.semantic_changes {
        counters.semantic_changes_inspected += 1;
        contains_opaque |= change.kind() == AuthoritativeAspectChangeKind::Opaque;
    }
    contains_opaque
}

fn require_opaque_widening_admission(
    contains_opaque: bool,
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
) -> Result<(), BridgeRouteError> {
    if !contains_opaque
        || admitted_widening == Some(BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect)
    {
        return Ok(());
    }
    Err(BridgeRouteError::new(
        BridgeRouteErrorKind::UnsupportedAuthoritativePatchPrecision,
        "opaque authoritative change has no admitted field or whole-aspect widening",
    ))
}

fn consume_record_semantic_changes(
    record: &PublishedAuthoritativeRecordPatch,
    remaining: &mut HashMap<ExpectedChange, usize>,
    counters: &mut BridgeAuthoritativePatchLoweringCounters,
) -> Result<(), BridgeRouteError> {
    for change in &record.semantic_changes {
        counters.semantic_changes_matched += 1;
        if change.precision() != PublishedAspectChangePrecision::Exact {
            return Err(denial(
                "Relational publication claimed widening before Bridge admission",
            ));
        }
        let matched = semantic_operation_candidates(change, record.structural_change)
            .into_iter()
            .any(|candidate| consume_expected(remaining, candidate));
        if !matched {
            let remaining = remaining
                .iter()
                .map(|(change, count)| (change.clone(), *count))
                .collect::<BTreeMap<_, _>>();
            return Err(denial(format!(
                "semantic change was not justified by the canonical authoritative patch: change={change:?}; remaining={remaining:?}"
            )));
        }
    }
    Ok(())
}

fn require_complete_semantic_coverage(
    remaining: &HashMap<ExpectedChange, usize>,
) -> Result<(), BridgeRouteError> {
    if !remaining.is_empty() {
        return Err(denial(
            "canonical authoritative patch operation had no semantic change",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExpectedChange {
    aspect_key: AspectKey,
    aspect_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    binding: AspectBinding,
    operation: ExpectedOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ExpectedOperation {
    WholeSet,
    WholeClear,
    FieldSet(CanonicalFieldPath),
    FieldClear(CanonicalFieldPath),
}

fn expected_changes(record: &PublishedAuthoritativeRecordPatch) -> Vec<ExpectedChange> {
    record
        .authoritative_patch
        .full_grammar_operations()
        .iter()
        .flat_map(|operation| match operation {
            PublishedAuthoritativePatchOperation::WholeAspectSet {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                ..
            } => vec![expected(
                aspect_key,
                *aspect_identity,
                *contract_revision,
                binding,
                ExpectedOperation::WholeSet,
            )],
            PublishedAuthoritativePatchOperation::WholeAspectClear {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
            } => vec![expected(
                aspect_key,
                *aspect_identity,
                *contract_revision,
                binding,
                ExpectedOperation::WholeClear,
            )],
            PublishedAuthoritativePatchOperation::FieldLevelPatch {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                field_sets,
                field_clears,
            } => field_sets
                .iter()
                .map(|field| {
                    expected(
                        aspect_key,
                        *aspect_identity,
                        *contract_revision,
                        binding,
                        ExpectedOperation::FieldSet(CanonicalFieldPath::single(
                            field.field.clone(),
                        )),
                    )
                })
                .chain(field_clears.iter().map(|field| {
                    expected(
                        aspect_key,
                        *aspect_identity,
                        *contract_revision,
                        binding,
                        ExpectedOperation::FieldClear(CanonicalFieldPath::single(field.clone())),
                    )
                }))
                .collect(),
        })
        .collect()
}

fn expected(
    aspect_key: &AspectKey,
    aspect_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    binding: &AspectBinding,
    operation: ExpectedOperation,
) -> ExpectedChange {
    ExpectedChange {
        aspect_key: aspect_key.clone(),
        aspect_identity,
        contract_revision,
        binding: binding.clone(),
        operation,
    }
}

fn semantic_operation_candidates(
    change: &PublishedAuthoritativeAspectChange,
    structural_change: RecordStructuralChange,
) -> Vec<ExpectedChange> {
    use AuthoritativeAspectChangeKind as Kind;
    let operations = match (change.binding(), change.kind(), change.field_path()) {
        (binding, Kind::FieldSet, Some(path)) if is_field_binding(binding) => {
            vec![ExpectedOperation::FieldSet(path.clone())]
        }
        (binding, Kind::FieldClear, Some(path)) if is_field_binding(binding) => {
            vec![ExpectedOperation::FieldClear(path.clone())]
        }
        (
            AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. },
            Kind::WholeAspectSet,
            None,
        ) => vec![ExpectedOperation::WholeSet],
        (
            AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. },
            Kind::WholeAspectClear,
            None,
        ) => vec![ExpectedOperation::WholeClear],
        (
            AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. },
            Kind::Opaque,
            None,
        )
        | (AspectBinding::RelationSourceEndpoint, Kind::RelationSourceEndpoint, None)
        | (AspectBinding::RelationTargetEndpoint, Kind::RelationTargetEndpoint, None) => {
            vec![ExpectedOperation::WholeSet, ExpectedOperation::WholeClear]
        }
        (
            AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet,
            kind,
            None,
        ) if kind == structural_kind(structural_change) => vec![ExpectedOperation::WholeSet],
        (AspectBinding::LifecycleTransition, kind, None)
            if Some(kind) == lifecycle_kind(structural_change) =>
        {
            vec![ExpectedOperation::WholeSet]
        }
        _ => Vec::new(),
    };
    operations
        .into_iter()
        .map(|operation| ExpectedChange {
            aspect_key: change.aspect_key().clone(),
            aspect_identity: change.aspect_identity(),
            contract_revision: change.contract_revision(),
            binding: change.binding().clone(),
            operation,
        })
        .collect()
}

fn consume_expected(
    remaining: &mut HashMap<ExpectedChange, usize>,
    candidate: ExpectedChange,
) -> bool {
    let Some(count) = remaining.get_mut(&candidate) else {
        return false;
    };
    *count -= 1;
    if *count == 0 {
        remaining.remove(&candidate);
    }
    true
}

fn lifecycle_kind(change: RecordStructuralChange) -> Option<AuthoritativeAspectChangeKind> {
    match change {
        RecordStructuralChange::Created => Some(AuthoritativeAspectChangeKind::LifecycleCreate),
        RecordStructuralChange::Deleted => Some(AuthoritativeAspectChangeKind::LifecycleDelete),
        RecordStructuralChange::RetainedForAudit => {
            Some(AuthoritativeAspectChangeKind::LifecycleRetainForAudit)
        }
        RecordStructuralChange::Updated => None,
        RecordStructuralChange::MaterializationSuspended
        | RecordStructuralChange::Rematerialized => None,
    }
}

fn structural_kind(change: RecordStructuralChange) -> AuthoritativeAspectChangeKind {
    match change {
        RecordStructuralChange::Created => AuthoritativeAspectChangeKind::StructuralCreate,
        RecordStructuralChange::Updated => AuthoritativeAspectChangeKind::StructuralUpdate,
        RecordStructuralChange::Deleted => AuthoritativeAspectChangeKind::StructuralDelete,
        RecordStructuralChange::RetainedForAudit => {
            AuthoritativeAspectChangeKind::StructuralRetainForAudit
        }
        RecordStructuralChange::MaterializationSuspended => {
            AuthoritativeAspectChangeKind::StructuralMaterializationSuspended
        }
        RecordStructuralChange::Rematerialized => {
            AuthoritativeAspectChangeKind::StructuralRematerialized
        }
    }
}

fn is_field_binding(binding: &AspectBinding) -> bool {
    matches!(
        binding,
        AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. }
    )
}

fn retain_emitted_target_counters(
    envelope: &PublishedAuthoritativePatchEnvelope,
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
    counters: &mut BridgeAuthoritativePatchLoweringCounters,
) {
    for record in &envelope.authoritative_record_patches {
        for change in &record.semantic_changes {
            counters.semantic_changes_emission_classified += 1;
            use AuthoritativeAspectChangeKind as Kind;
            match change.kind() {
                Kind::FieldSet | Kind::FieldClear => counters.field_targets_emitted += 1,
                Kind::WholeAspectSet | Kind::WholeAspectClear => {
                    counters.whole_aspect_targets_emitted += 1
                }
                Kind::RelationSourceEndpoint | Kind::RelationTargetEndpoint => {
                    counters.endpoint_targets_emitted += 1
                }
                Kind::LifecycleCreate | Kind::LifecycleDelete | Kind::LifecycleRetainForAudit => {
                    counters.lifecycle_targets_emitted += 1
                }
                Kind::Opaque => {
                    counters.opaque_changes_emitted += 1;
                    if admitted_widening
                        == Some(BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect)
                    {
                        counters.declared_widenings += 1;
                        counters.whole_aspect_targets_emitted += 1;
                    }
                }
                Kind::StructuralCreate
                | Kind::StructuralUpdate
                | Kind::StructuralDelete
                | Kind::StructuralRetainForAudit
                | Kind::StructuralMaterializationSuspended
                | Kind::StructuralRematerialized => {}
            }
        }
    }
}

fn denial(detail: impl Into<String>) -> BridgeRouteError {
    let detail: String = detail.into();
    BridgeRouteError::new(
        BridgeRouteErrorKind::InvalidAuthoritativePatchSemantics,
        detail,
    )
}
