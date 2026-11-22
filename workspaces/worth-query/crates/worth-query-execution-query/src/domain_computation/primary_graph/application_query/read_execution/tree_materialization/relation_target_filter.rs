use std::collections::BTreeSet;

use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_installation::facade::WorthQueryInstalledGraphRelation;
use worth_relational::facade::{
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope},
    storage::RecordLifecycleState,
};

use super::{projection_denial, ResultTreeWork, WorthQueryApplicationReadExecutionDenial};
use crate::domain_computation::primary_graph::{
    application_query::disclosure::WorthQueryApplicationQueryGovernance,
    WorthQueryPrimaryGraphLayout,
};

pub fn retain_matching_targets<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView,
    graph: &WorthQueryPrimaryGraphLayout,
    governance: &WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    targets: &mut Vec<T>,
    entity: impl Fn(&T) -> worth_relational::facade::identity::EntityId,
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let Some(predicate) = relation.predicate() else {
        return Ok(());
    };
    let expected = predicate
        .parameter()
        .map(|parameter| {
            parameters
                .bindings()
                .iter()
                .find(|(name, _)| name.as_ref() == parameter)
                .map(|(_, value)| value)
                .filter(|value| value.value_family() == predicate.scalar_family())
                .ok_or_else(|| projection_denial(parameter))
        })
        .transpose()?;
    let child_kind = graph
        .entity_kind(relation.child_entity())
        .ok_or_else(|| projection_denial(relation.child_entity()))?;
    let Some(field) = predicate.field() else {
        let Some(worth_foundational::facade::AspectValue::EntityRef(expected)) = expected else {
            return Err(projection_denial(relation.result_path()));
        };
        return retain_exact_entity_targets(
            execution, projection, relation, targets, entity, child_kind, *expected, work,
        );
    };
    let (aspect_key, field_key) = predicate
        .field_keys()
        .ok_or_else(|| projection_denial(relation.result_path()))?;
    let admission = governance
        .admit_internal_projection(
            field,
            field_key,
            ApplicationQueryObservableInfluence::RowPresence,
        )
        .ok_or_else(|| projection_denial(field.2))?;
    let layout = graph
        .equality_field(field.0, field.1, field.2)
        .filter(|layout| {
            layout.entity_kind == child_kind && admission.admits_locator(&layout.locator)
        })
        .ok_or_else(|| projection_denial(field.2))?;
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        aspect_key.clone(),
        BTreeSet::from([field_key.clone()]),
    )]);
    let candidates = std::mem::take(targets);
    for target in candidates {
        work.charge_relation_predicate(relation.result_path())?;
        let matches = projection
            .entity_record_with_projection_scope(execution, entity(&target), scope.clone(), |record| {
                (record.kind_id() == layout.entity_kind
                    && record.lifecycle() == RecordLifecycleState::Live)
                    .then(|| {
                        work.reads.record_fields(1);
                        record.aspect_field_value(aspect_key, field_key)
                            == expected
                    })
            }).map_err(|cause| crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(cause, relation.result_path()))?
            .ok_or_else(|| projection_denial(relation.result_path()))?;
        if matches {
            work.material_next(targets)?;
            targets.push(target);
        }
    }
    Ok(())
}

fn retain_exact_entity_targets<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView,
    relation: &WorthQueryInstalledGraphRelation,
    targets: &mut Vec<T>,
    entity: impl Fn(&T) -> worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    expected: worth_foundational::facade::EntityId,
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    // Only actual relation endpoints are projected. The external coordinate is
    // compared as DATA, including generation zero; it never selects a Native slot.
    for target in std::mem::take(targets) {
        work.charge_relation_predicate(relation.result_path())?;
        let matches = projection
            .entity_record_with_projection_scope(
                execution,
                entity(&target),
                ProjectionAspectScope::empty(),
                |record| {
                    let actual = record.entity_id();
                    (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live)
                        .then_some(
                            actual.partition_value() == expected.partition_id.0
                                && actual.local_slot_value() == expected.local_slot.0
                                && actual.generation_value() == expected.generation.0,
                        )
                },
            )
            .map_err(|cause| {
                super::super::denial::relational_read_denial(cause, relation.result_path())
            })?
            .ok_or_else(|| projection_denial(relation.result_path()))?;
        if matches {
            work.material_next(targets)?;
            targets.push(target);
        }
    }
    Ok(())
}
