use std::collections::HashMap;

use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey,
    AuthoritativeAspectChangeKind, CanonicalFieldPath,
};

use crate::publication::patch::data::{
    PublishedAuthoritativeAspectChange, PublishedAuthoritativePatchOperation,
    PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};

/// One change a record's canonical patch operations call for. A published
/// semantic change must match one of these, and every one must be matched.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ExpectedChange {
    aspect: ExpectedAspect,
    operation: ExpectedOperation,
}

/// The aspect an operation or a semantic change names.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExpectedAspect {
    aspect_key: AspectKey,
    aspect_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    binding: AspectBinding,
}

impl ExpectedAspect {
    fn of_operation(operation: &PublishedAuthoritativePatchOperation) -> Self {
        use PublishedAuthoritativePatchOperation as Operation;
        match operation {
            Operation::WholeAspectSet {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                ..
            }
            | Operation::WholeAspectClear {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
            }
            | Operation::FieldLevelPatch {
                aspect_key,
                aspect_identity,
                contract_revision,
                binding,
                ..
            } => Self {
                aspect_key: aspect_key.clone(),
                aspect_identity: *aspect_identity,
                contract_revision: *contract_revision,
                binding: binding.clone(),
            },
        }
    }

    fn of_change(change: &PublishedAuthoritativeAspectChange) -> Self {
        Self {
            aspect_key: change.aspect_key().clone(),
            aspect_identity: change.aspect_identity(),
            contract_revision: change.contract_revision(),
            binding: change.binding().clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ExpectedOperation {
    WholeSet,
    WholeClear,
    FieldSet(CanonicalFieldPath),
    FieldClear(CanonicalFieldPath),
}

pub(super) fn expected_changes(record: &PublishedAuthoritativeRecordPatch) -> Vec<ExpectedChange> {
    record
        .authoritative_patch
        .full_grammar_operations()
        .iter()
        .flat_map(|operation| {
            let aspect = ExpectedAspect::of_operation(operation);
            expected_operations(operation)
                .into_iter()
                .map(move |operation| ExpectedChange {
                    aspect: aspect.clone(),
                    operation,
                })
        })
        .collect()
}

fn expected_operations(operation: &PublishedAuthoritativePatchOperation) -> Vec<ExpectedOperation> {
    match operation {
        PublishedAuthoritativePatchOperation::WholeAspectSet { .. } => {
            vec![ExpectedOperation::WholeSet]
        }
        PublishedAuthoritativePatchOperation::WholeAspectClear { .. } => {
            vec![ExpectedOperation::WholeClear]
        }
        PublishedAuthoritativePatchOperation::FieldLevelPatch {
            field_sets,
            field_clears,
            ..
        } => field_sets
            .iter()
            .map(|set| ExpectedOperation::FieldSet(CanonicalFieldPath::single(set.field.clone())))
            .chain(field_clears.iter().map(|field| {
                ExpectedOperation::FieldClear(CanonicalFieldPath::single(field.clone()))
            }))
            .collect(),
    }
}

pub(super) fn semantic_operation_candidates(
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
    let aspect = ExpectedAspect::of_change(change);
    operations
        .into_iter()
        .map(|operation| ExpectedChange {
            aspect: aspect.clone(),
            operation,
        })
        .collect()
}

pub(super) fn consume_expected(
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
