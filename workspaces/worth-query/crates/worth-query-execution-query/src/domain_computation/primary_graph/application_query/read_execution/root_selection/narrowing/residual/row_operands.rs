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
use worth_foundational::facade::AspectValue;
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

pub struct RowOperandReader<'filter> {
    fields: &'filter [WorthQueryInstalledWhereFilterField],
    kind: KindId,
    scope: ProjectionAspectScope,
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
        let scope_work = (field_count as u64)
            .checked_mul(log2_ceil(field_count))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        charge_work(read.execution, work, scope_work)?;
        allocation
            .claim_tree_copy::<(
                worth_foundational::facade::AspectKey,
                BTreeSet<worth_foundational::facade::FieldKey>,
            )>(read.execution, field_count)
            .map_err(stop)?;
        allocation
            .claim_tree_copy::<worth_foundational::facade::FieldKey>(read.execution, field_count)
            .map_err(stop)?;
        allocation
            .claim_slice::<ProjectionAspectRequirement>(read.execution, field_count)
            .map_err(stop)?;
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
        Ok(Self {
            fields: filter.fields(),
            kind,
            scope,
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
        let scope_clone_bytes = u64::try_from(self.scope.owned_allocation_capacity_bytes())
            .map_err(|_| stop(Cause::ChargedBytesOverflow))?;
        if scope_clone_bytes > 0 {
            row_alloc.claim(read.execution, scope_clone_bytes).map_err(stop)?;
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
                            let mut quote_bytes = (self.fields.len()
                                * size_of::<(String, ExpressionValue)>())
                                as u64;
                            for field in self.fields {
                                let observed = record
                                    .observe_aspect_field(field.aspect_key(), field.field_key());
                                let optional =
                                    field.presence() == ApplicationFieldPresence::Optional;
                                let (present, payload) = match observed {
                                    Some(Observation::Present(value)) => {
                                        (true, payload_bytes(value, false))
                                    }
                                    Some(Observation::FieldAbsent | Observation::AspectAbsent)
                                        if optional =>
                                    {
                                        (false, 0)
                                    }
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
