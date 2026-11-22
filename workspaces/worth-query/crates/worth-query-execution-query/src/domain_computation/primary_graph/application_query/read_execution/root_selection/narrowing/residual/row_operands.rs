//! One candidate's filter field operands, observed and read exactly once.
use super::{projection_unavailable, NarrowingRead, RootPathSourceBuilder, RootSelectionWork};
use crate::domain_computation::primary_graph::application_query::read_execution::{
    relational_read_denial, WorthQueryApplicationReadExecutionDenial,
};
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

pub struct RowOperandReader<'filter> {
    fields: &'filter [WorthQueryInstalledWhereFilterField],
    kind: KindId,
    scope: ProjectionAspectScope,
}

impl<'filter> RowOperandReader<'filter> {
    pub fn new<ReadIdentity>(
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        filter: &'filter WorthQueryInstalledWhereFilter,
    ) -> Result<Self, WorthQueryApplicationReadExecutionDenial> {
        let kind = read
            .graph
            .entity_kind(read.plan.contract.root_entity())
            .ok_or_else(projection_unavailable)?;
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
    /// live root record once. Absent optional fields are `none`.
    pub fn read<ReadIdentity>(
        &self,
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        candidate: EntityId,
        evidence: &mut RootPathSourceBuilder,
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
                            self.fields
                                .iter()
                                .map(|field| {
                                    let observed = record
                                        .observe_aspect_field(field.aspect_key(), field.field_key());
                                    Ok((field.operand().to_owned(), operand(field, observed)?))
                                })
                                .collect::<Result<Vec<_>, WorthQueryApplicationReadExecutionDenial>>()
                        })
                },
            )
            .map_err(|cause| relational_read_denial(cause, super::WHERE_FILTER_SUBJECT))?
            .ok_or_else(projection_unavailable)??;
        work.charge_predicate(1, 0, self.fields.len(), super::WHERE_FILTER_SUBJECT)?;
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
