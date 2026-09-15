use std::collections::BTreeSet;

use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_installation::facade::WorthQueryInstalledGraphRelation;
use worth_relational::facade::{
    identity::EntityId,
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope},
    storage::RecordLifecycleState,
};

use super::{projection_denial, ResultTreeWork, WorthQueryApplicationReadExecutionDenial};
use crate::domain_computation::primary_graph::{
    application_query::disclosure::WorthQueryApplicationQueryGovernance,
    WorthQueryPrimaryGraphLayout,
};

pub(super) fn retain_matching_targets(
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    graph: &WorthQueryPrimaryGraphLayout,
    governance: &WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    targets: &mut Vec<EntityId>,
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let Some(predicate) = relation.predicate() else {
        return Ok(());
    };
    let expected = parameters
        .bindings()
        .iter()
        .find(|(name, _)| *name == predicate.parameter())
        .map(|(_, value)| value)
        .ok_or_else(|| projection_denial(predicate.parameter()))?;
    if expected.value_family() != predicate.scalar_family() {
        return Err(projection_denial(predicate.parameter()));
    }
    let field = predicate.field();
    let admission = governance
        .admit_internal_projection(
            field,
            predicate.field_key(),
            ApplicationQueryObservableInfluence::RowPresence,
        )
        .ok_or_else(|| projection_denial(field.2))?;
    let child_kind = graph
        .entity_kind(relation.child_entity())
        .ok_or_else(|| projection_denial(relation.child_entity()))?;
    let layout = graph
        .equality_field(field.0, field.1, field.2)
        .filter(|layout| {
            layout.entity_kind == child_kind && admission.admits_locator(&layout.locator)
        })
        .ok_or_else(|| projection_denial(field.2))?;
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        predicate.aspect_key().clone(),
        BTreeSet::from([predicate.field_key().clone()]),
    )]);
    let candidates = std::mem::take(targets);
    for entity_id in candidates {
        work.charge_relation_predicate(relation.result_path())?;
        let matches = projection
            .entity_record_with_projection_scope(entity_id, scope.clone(), |record| {
                (record.kind_id() == layout.entity_kind
                    && record.lifecycle() == RecordLifecycleState::Live)
                    .then(|| {
                        record.aspect_field_value(predicate.aspect_key(), predicate.field_key())
                            == Some(expected)
                    })
            })
            .ok_or_else(|| projection_denial(relation.result_path()))?;
        if matches {
            targets.push(entity_id);
        }
    }
    Ok(())
}
