//! One candidate's filter field operands, observed and read exactly once.
use super::{
    charge_work, log2_ceil, payload_bytes, projection_unavailable, try_vec_exact, NarrowingRead,
    RootPathSourceBuilder, RootSelectionWork, WHERE_FILTER_SUBJECT,
};
use crate::domain_computation::primary_graph::application_query::read_execution::{
    relational_read_denial, WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::expression_api::ExpressionValue;
use worth_foundational::facade::{
    AspectKey, AspectValue, CanonicalBasisEntry, CanonicalDigestPreparationEntry,
    CanonicalFieldPath, FieldKey,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
use worth_query_installation::facade::{
    WorthQueryInstalledWhereFilter, WorthQueryInstalledWhereFilterField,
};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{
    ProjectionAspectRequirement, ProjectionAspectScope,
    RelationalProjectedFieldObservation as Observation,
};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

/// Exact byte length of `"worth.relational.visibility.projection_scope.v1"`.
const SCOPE_CANONICAL_VERSION_BYTES: usize = 47;
/// Exact byte length of `".projection.field."` in canonical mask loci.
const SCOPE_CANONICAL_LOCUS_SEPARATOR_BYTES: usize = 18;

pub struct RowOperandReader<'filter> {
    fields: &'filter [WorthQueryInstalledWhereFilterField],
    kind: KindId,
    scope: ProjectionAspectScope,
    observe_work: u64,
}

impl<'filter> RowOperandReader<'filter> {
    pub fn new<ReadIdentity>(
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        filter: &'filter WorthQueryInstalledWhereFilter,
        allocation: &SourceFactsAllocation,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<Self, WorthQueryApplicationReadExecutionDenial> {
        let stop = |cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT);
        let kind = read
            .graph
            .entity_kind(read.plan.contract.root_entity())
            .ok_or_else(projection_unavailable)?;
        let field_count = filter.fields().len();
        let setup_bytes = setup_scope_payload_bytes(filter.fields())
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        let scope_work = (field_count as u64)
            .checked_mul(log2_ceil(field_count))
            .and_then(|n| n.checked_add(setup_bytes))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        charge_work(read.execution, work, scope_work)?;
        allocation
            .claim_tree_copy::<(AspectKey, BTreeSet<FieldKey>)>(read.execution, field_count)
            .map_err(stop)?;
        allocation
            .claim_tree_copy::<FieldKey>(read.execution, field_count)
            .map_err(stop)?;
        allocation
            .claim_slice::<ProjectionAspectRequirement>(read.execution, field_count)
            .map_err(stop)?;
        if setup_bytes > 0 {
            allocation
                .claim(read.execution, setup_bytes)
                .map_err(stop)?;
        }
        let mut keys = BTreeMap::new();
        for field in filter.fields() {
            keys.entry(field.aspect_key().clone())
                .or_insert_with(BTreeSet::new)
                .insert(field.field_key().clone());
        }
        let scope = ProjectionAspectScope::from_requirements(
            keys.into_iter()
                .map(|(aspect, fields)| ProjectionAspectRequirement::fields(aspect, fields)),
        );
        let roster_width = read.projection.entity_aspect_binding_count(kind);
        let observe_work = observe_row_work(field_count, &scope, roster_width)
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        Ok(Self {
            fields: filter.fields(),
            kind,
            scope,
            observe_work,
        })
    }

    /// Observes every filter field's revision as set evidence, then reads the
    /// live root record once under `row_alloc`. Absent optional fields are `none`.
    pub fn read<ReadIdentity>(
        &self,
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        candidate: EntityId,
        evidence: &mut RootPathSourceBuilder,
        row_alloc: &SourceFactsAllocation,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<Vec<(String, ExpressionValue)>, WorthQueryApplicationReadExecutionDenial> {
        evidence.observe_entity(read.execution, candidate)?;
        for field in self.fields {
            evidence.observe_guard_field(
                read.execution,
                read.projection,
                read.graph,
                candidate,
                read.plan.contract.root_entity(),
                field.aspect_key(),
                field.field_key(),
                work,
            )?;
        }
        let stop = |cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT);
        charge_work(read.execution, work, self.observe_work)?;
        let scope_clone_bytes = u64::try_from(self.scope.owned_allocation_capacity_bytes())
            .map_err(|_| stop(Cause::ChargedBytesOverflow))?;
        if scope_clone_bytes > 0 {
            row_alloc
                .claim(read.execution, scope_clone_bytes)
                .map_err(stop)?;
        }
        let mut row_work = 0_u64;
        let values = read
            .projection
            .entity_record_with_projection_scope(
                read.execution,
                candidate,
                self.scope.clone(),
                |record| {
                    (record.kind_id() == self.kind
                        && record.lifecycle() == RecordLifecycleState::Live)
                        .then(|| {
                            let mut quote_work = 0_u64;
                            let mut quote_bytes =
                                (self.fields.len() * size_of::<(String, ExpressionValue)>()) as u64;
                            for field in self.fields {
                                let observed = record
                                    .observe_aspect_field(field.aspect_key(), field.field_key());
                                let optional =
                                    field.presence() == ApplicationFieldPresence::Optional;
                                let (present, payload) =
                                    match observed {
                                        Some(Observation::Present(value)) => {
                                            (true, payload_bytes(value, false))
                                        }
                                        Some(
                                            Observation::FieldAbsent | Observation::AspectAbsent,
                                        ) if optional => (false, 0),
                                        _ => return Err(projection_unavailable()),
                                    };
                                let slot_bytes = (field.operand().len()
                                    + if present {
                                        size_of::<ExpressionValue>() + 2 * size_of::<usize>()
                                    } else {
                                        0
                                    }
                                    + if optional && present {
                                        size_of::<ExpressionValue>()
                                    } else {
                                        0
                                    }) as u64;
                                quote_work = quote_work
                                    .checked_add(1)
                                    .and_then(|n| n.checked_add(payload))
                                    .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
                                quote_bytes = quote_bytes
                                    .checked_add(slot_bytes)
                                    .and_then(|n| n.checked_add(payload))
                                    .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
                            }
                            if quote_work > 0 {
                                SourceFactsAllocation::work(read.execution, quote_work)
                                    .map_err(stop)?;
                            }
                            if quote_bytes > 0 {
                                row_alloc.claim(read.execution, quote_bytes).map_err(stop)?;
                            }
                            row_work = quote_work;
                            let mut out = try_vec_exact(self.fields.len())?;
                            for field in self.fields {
                                let observed = record
                                    .observe_aspect_field(field.aspect_key(), field.field_key());
                                out.push((field.operand().to_owned(), operand(field, observed)?));
                            }
                            Ok(out)
                        })
                },
            )
            .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?
            .ok_or_else(projection_unavailable)??;
        if row_work > 0 {
            work.charge_residual_filter(
                usize::try_from(row_work).unwrap_or(usize::MAX),
                WHERE_FILTER_SUBJECT,
            )?;
        }
        work.charge_predicate(1, 0, self.fields.len(), WHERE_FILTER_SUBJECT)?;
        Ok(values)
    }
}

fn operand(
    field: &WorthQueryInstalledWhereFilterField,
    observed: Option<Observation<'_>>,
) -> Result<ExpressionValue, WorthQueryApplicationReadExecutionDenial> {
    let optional = field.presence() == ApplicationFieldPresence::Optional;
    match observed {
        Some(Observation::Present(value)) => {
            let value = scalar(field, value)?;
            Ok(if optional {
                ExpressionValue::some(value)
            } else {
                value
            })
        }
        Some(Observation::FieldAbsent | Observation::AspectAbsent) if optional => {
            Ok(ExpressionValue::none())
        }
        _ => Err(projection_unavailable()),
    }
}

fn scalar(
    field: &WorthQueryInstalledWhereFilterField,
    value: &AspectValue,
) -> Result<ExpressionValue, WorthQueryApplicationReadExecutionDenial> {
    if value.value_family() != field.scalar_family() {
        return Err(projection_unavailable());
    }
    ExpressionValue::from_aspect_value(value).map_err(|_| projection_unavailable())
}

/// Upper-bound heap bytes for `keys` strings, `AspectMask` paths, copied
/// `AspectMaskLocator` paths, and `prepare_projection_mask_for_canonical_basis`
/// (including version strings, digest entries, and canonical basis sequence).
fn setup_scope_payload_bytes(fields: &[WorthQueryInstalledWhereFilterField]) -> Option<u64> {
    let mut total = 0_usize;
    let per_field_fixed = 3_usize
        .checked_mul(size_of::<CanonicalFieldPath>().checked_add(size_of::<FieldKey>())?)?
        .checked_add(size_of::<CanonicalDigestPreparationEntry>())?
        .checked_add(size_of::<CanonicalBasisEntry>().checked_mul(2)?)?
        .checked_add(size_of::<&str>())?
        .checked_add(SCOPE_CANONICAL_VERSION_BYTES.checked_mul(2)?)?
        .checked_add(SCOPE_CANONICAL_LOCUS_SEPARATOR_BYTES.checked_mul(2)?)?;
    for field in fields {
        let aspect_len = field.aspect_key().as_str().len();
        let field_len = field.field_key().as_str().len();
        // Owned key copies across: keys map (1), locator (1), canonical prep
        // aspect clone (1), digest entry (1), canonical locus (1).
        let aspect_bytes = aspect_len.checked_mul(5)?;
        // Owned field copies across: keys/mask (1), locator (1), canonical prep
        // mask clone (1), digest entry (1), path_text + locus + exact_text +
        // transient locus format (4).
        let field_bytes = field_len.checked_mul(8)?;
        total = total
            .checked_add(per_field_fixed)?
            .checked_add(aspect_bytes)?
            .checked_add(field_bytes)?;
    }
    u64::try_from(total).ok()
}

/// Per-candidate projection-scope admission and two-pass field-observation
/// work, explicitly scaled by the root entity kind's executable binding roster
/// width (`admit_declared_projection_aspects` + both `observe_aspect_field`
/// passes).
fn observe_row_work(
    field_count: usize,
    scope: &ProjectionAspectScope,
    roster_width: usize,
) -> Option<u64> {
    let roster = roster_width.max(1);
    let roster_u64 = u64::try_from(roster).ok()?;
    let scope_reqs = u64::try_from(scope.requirements().len()).ok()?;
    let mut scope_scan_steps = 0_u64;
    for req in scope.requirements() {
        let paths = u64::try_from(req.mask().paths().len()).ok()?;
        scope_scan_steps = scope_scan_steps.checked_add(paths.checked_add(1)?)?;
    }
    let admit_work = scope_reqs
        .checked_mul(roster_u64)?
        .checked_add(scope_scan_steps)?;
    let per_observe_work = scope_scan_steps
        .max(1)
        .checked_add(roster_u64)?
        .checked_add(log2_ceil(roster))?;
    u64::try_from(field_count)
        .ok()?
        .checked_mul(per_observe_work)?
        .checked_mul(2)?
        .checked_add(admit_work)?
        .checked_add(scope_reqs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_row_work_scales_with_root_kind_roster_width_for_fixed_scope() {
        let aspect = AspectKey::new("ItemCore").unwrap();
        let field = FieldKey::new("Tag").unwrap();
        let scope = ProjectionAspectScope::fields(aspect, [field]);
        let narrow = observe_row_work(1, &scope, 1).unwrap();
        let wide = observe_row_work(1, &scope, 16).unwrap();
        // 1 requirement * (16 - 1) for admission + 2 passes * ((16 - 1) + (log2_ceil(16) - log2_ceil(1))).
        let expected_delta = 3 * (16 - 1) + 2 * (log2_ceil(16) - log2_ceil(1));
        assert_eq!(wide - narrow, expected_delta);
    }
}
