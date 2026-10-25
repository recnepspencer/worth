use crate::merge::data::{MergeConflictClassification, VisibleMergeRecord};
use crate::merge::policy::contexts::{PolicyBaseReadViewContext, PolicyReadViewContext};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use worth_foundational::facade::{
    AspectKey, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
};

use super::base_commit_patch_basis::scalar_from_base_commit_patch;
use super::source_basis::PolicyValueSourceBasis;
use super::visible_record_basis::{
    entity_basis_for_visible_record, entity_basis_record_refs, relation_basis_for_visible_record,
    relation_basis_record_refs,
};
use super::{
    foundational_policy_value_transition_basis, PolicyAspectValueBasis, PolicyScalarValue,
    PolicyValueLookupCounters, PolicyValueLookupFailure, PolicyValueLookupReceipt,
    PolicyValueProvenance, ScalarPolicyAspectBinding,
};

pub(crate) fn resolve_policy_aspect_value_basis(
    record: &VisibleMergeRecord,
    classification: &MergeConflictClassification,
    binding: ScalarPolicyAspectBinding,
    source_branch: &crate::history::data::BranchId,
    target_branch: &crate::history::data::BranchId,
    base_commit_id: crate::history::data::CommitId,
    source_view: &PolicyReadViewContext<'_>,
    target_view: &PolicyReadViewContext<'_>,
    base_view: &PolicyBaseReadViewContext<'_>,
) -> PolicyAspectValueBasis {
    let mut counters = PolicyValueLookupCounters::default();
    let source = source_scalar_value(record, &binding, source_view);
    counters.record_source(source.as_ref().map(|_| ()).map_err(|failure| *failure));
    let target = target_scalar_value(record, classification, &binding, target_view);
    counters.record_target(target.as_ref().map(|_| ()).map_err(|failure| *failure));
    let base = base_scalar_value(
        record,
        classification,
        &binding,
        base_commit_id,
        base_view,
        &mut counters,
    );
    let receipt = PolicyValueLookupReceipt::from_counters(counters);
    let (merge_basis, merge_base_selection_basis, strategy_basis) =
        foundational_policy_value_transition_basis(
            source_branch,
            target_branch,
            binding.aspect_key(),
            base_commit_id,
        );

    PolicyAspectValueBasis::new(
        binding,
        merge_basis,
        merge_base_selection_basis,
        strategy_basis,
        source,
        target,
        base,
        receipt,
    )
}

fn source_scalar_value(
    record: &VisibleMergeRecord,
    binding: &ScalarPolicyAspectBinding,
    source_view: &PolicyReadViewContext<'_>,
) -> Result<PolicyScalarValue, PolicyValueLookupFailure> {
    match binding {
        ScalarPolicyAspectBinding::Entity { aspect_key } => {
            let entity_id = record
                .record_ref
                .entity_id()
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            let entity = source_view
                .entity_for_record(entity_id, record.source_lineage_id.or(record.lineage_id))
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            scalar_from_entity(
                entity,
                aspect_key,
                PolicyValueProvenance::SourceVisibleState,
            )
        }
        ScalarPolicyAspectBinding::Relation { aspect_key } => {
            let relation_id = record
                .record_ref
                .relation_id()
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            let relation = source_view
                .relation_for_record(relation_id)
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            scalar_from_relation(
                relation,
                aspect_key,
                PolicyValueProvenance::SourceVisibleState,
            )
        }
    }
}

fn target_scalar_value(
    record: &VisibleMergeRecord,
    classification: &MergeConflictClassification,
    binding: &ScalarPolicyAspectBinding,
    target_view: &PolicyReadViewContext<'_>,
) -> Result<PolicyScalarValue, PolicyValueLookupFailure> {
    let target_record_ref = classification
        .target_record
        .as_ref()
        .unwrap_or(&record.record_ref);
    match binding {
        ScalarPolicyAspectBinding::Entity { aspect_key } => {
            let entity_id = target_record_ref
                .entity_id()
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            let entity = target_view
                .entity_for_record(entity_id, record.target_lineage_id.or(record.lineage_id))
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            scalar_from_entity(
                entity,
                aspect_key,
                PolicyValueProvenance::TargetVisibleState,
            )
        }
        ScalarPolicyAspectBinding::Relation { aspect_key } => {
            let relation_id = target_record_ref
                .relation_id()
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            let relation = target_view
                .relation_for_record(relation_id)
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)?;
            scalar_from_relation(
                relation,
                aspect_key,
                PolicyValueProvenance::TargetVisibleState,
            )
        }
    }
}

fn base_scalar_value(
    record: &VisibleMergeRecord,
    classification: &MergeConflictClassification,
    binding: &ScalarPolicyAspectBinding,
    base_commit_id: crate::history::data::CommitId,
    base_view: &PolicyBaseReadViewContext<'_>,
    counters: &mut PolicyValueLookupCounters,
) -> Result<PolicyScalarValue, PolicyValueLookupFailure> {
    let aspect_key = binding.aspect_key();
    let candidate_targets = match binding {
        ScalarPolicyAspectBinding::Entity { .. } => {
            entity_basis_record_refs(record, classification)
        }
        ScalarPolicyAspectBinding::Relation { .. } => {
            relation_basis_record_refs(record, classification)
        }
    };
    let patch_value =
        scalar_from_base_commit_patch(base_view.envelope(), &candidate_targets, aspect_key).map(
            |value| {
                PolicyScalarValue::new(
                    value,
                    PolicyValueProvenance::BaseCommitPatch,
                    aspect_key,
                    PolicyValueSourceBasis::CommitPatch { base_commit_id },
                )
            },
        );
    counters.record_base_patch_authority(patch_value.is_some());
    if let Some(patch_value) = patch_value {
        return Ok(patch_value);
    }

    let base_state = match binding {
        ScalarPolicyAspectBinding::Entity { aspect_key } => {
            entity_basis_for_visible_record(record, classification, base_view.records())
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)
                .and_then(|entity| {
                    scalar_from_entity(entity, aspect_key, PolicyValueProvenance::BaseReadViewState)
                })
        }
        ScalarPolicyAspectBinding::Relation { aspect_key } => {
            relation_basis_for_visible_record(record, classification, base_view.records())
                .ok_or(PolicyValueLookupFailure::MissingRecordBasis)
                .and_then(|relation| {
                    scalar_from_relation(
                        relation,
                        aspect_key,
                        PolicyValueProvenance::BaseReadViewState,
                    )
                })
        }
    };
    counters.record_base_state(base_state.as_ref().map(|_| ()).map_err(|failure| *failure));
    base_state
}

fn scalar_from_entity(
    entity: &EntityReadRecord,
    aspect_key: &AspectKey,
    provenance: PolicyValueProvenance,
) -> Result<PolicyScalarValue, PolicyValueLookupFailure> {
    scalar_from_authoritative_state(
        entity.authoritative_aspect_state.as_ref(),
        aspect_key,
        provenance,
    )
}

fn scalar_from_relation(
    relation: &RelationReadRecord,
    aspect_key: &AspectKey,
    provenance: PolicyValueProvenance,
) -> Result<PolicyScalarValue, PolicyValueLookupFailure> {
    scalar_from_authoritative_state(
        relation.authoritative_aspect_state.as_ref(),
        aspect_key,
        provenance,
    )
}

fn scalar_from_authoritative_state(
    authoritative_state: Option<&AuthoritativeRecordAspectState>,
    aspect_key: &AspectKey,
    provenance: PolicyValueProvenance,
) -> Result<PolicyScalarValue, PolicyValueLookupFailure> {
    let Some(entry) = authoritative_state.and_then(|state| state.get(aspect_key)) else {
        return Err(PolicyValueLookupFailure::MissingField);
    };
    match entry.view() {
        ContractValidatedAspectValueView::Scalar(value) => Ok(PolicyScalarValue::new(
            value.clone(),
            provenance,
            aspect_key,
            PolicyValueSourceBasis::VisibleReadState,
        )),
        ContractValidatedAspectValueView::Struct(_) => {
            Err(PolicyValueLookupFailure::InvalidValueShape)
        }
    }
}

trait RecordRefValueBasisExt {
    fn entity_id(&self) -> Option<crate::identity::data::EntityId>;
    fn relation_id(&self) -> Option<crate::identity::data::RelationId>;
}

impl RecordRefValueBasisExt for crate::transactions::data::RecordRef {
    fn entity_id(&self) -> Option<crate::identity::data::EntityId> {
        match self {
            Self::Entity(entity_id) => Some(*entity_id),
            Self::Relation(_) => None,
        }
    }

    fn relation_id(&self) -> Option<crate::identity::data::RelationId> {
        match self {
            Self::Entity(_) => None,
            Self::Relation(relation_id) => Some(*relation_id),
        }
    }
}
