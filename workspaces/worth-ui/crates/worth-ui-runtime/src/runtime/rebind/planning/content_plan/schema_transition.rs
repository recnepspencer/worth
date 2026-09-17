use std::collections::BTreeSet;
use std::sync::Arc;

use crate::runtime::rebind::{
    UiProjectionSchemaRequirement, UiProjectionSchemaTransition, UiProjectionSchemaTransitionInput,
    UiProjectionSchemaTransitionKind,
};

pub(super) fn compile(
    predecessor: &crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationAuthority,
    candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    scope: &crate::runtime::rebind::UiResolvedAffectedScope,
    content: &mut crate::mounting::UiMountedSemanticContentInput,
) -> Result<BTreeSet<crate::graph::UiGraphNodeIdentity>, super::super::UiRebindPlanningDenial> {
    if predecessor.generation_identity() == candidate.generation_identity() {
        return Ok(BTreeSet::new());
    }
    let mut governed = BTreeSet::new();
    for edge in candidate.semantic_handoff().projection_contents() {
        let identity = edge.projection_identity();
        let Some(predecessor_requirement) = predecessor
            .semantic_handoff()
            .projection_requirement(identity)
            .and_then(schema_requirement)
        else {
            continue;
        };
        let Some(candidate_requirement) = candidate
            .semantic_handoff()
            .projection_requirement(identity)
            .and_then(schema_requirement)
        else {
            continue;
        };
        let Some(installed_requirement) =
            installed_requirement(candidate.query_binding_plan(), identity)
        else {
            continue;
        };
        let Some(kind) = classify_transition(
            &predecessor_requirement,
            &candidate_requirement,
            &installed_requirement,
        ) else {
            continue;
        };
        let mut correlated = 0usize;
        for consumer in scope
            .consumers()
            .iter()
            .filter(|consumer| consumer.key().authored_identity() == edge.component_identity())
        {
            let Some(crate::graph::UiGraphFactConsumerIdentity::GraphNode(graph_node)) =
                consumer.candidate().or(consumer.predecessor())
            else {
                continue;
            };
            insert_content(content, graph_node, &installed_requirement, kind)?;
            content.record_schema_transition(UiProjectionSchemaTransition::new(
                UiProjectionSchemaTransitionInput {
                    kind,
                    component_identity: edge.component_identity().into(),
                    declaration_identity: candidate
                        .semantic_handoff()
                        .projection_requirement(identity)
                        .expect("candidate requirement was resolved above")
                        .declaration_identity()
                        .into(),
                    view_identity: identity.clone(),
                    graph_node,
                    predecessor: predecessor_requirement.clone(),
                    candidate: candidate_requirement.clone(),
                    installed: installed_requirement.clone(),
                },
            ));
            governed.insert(graph_node);
            correlated += 1;
        }
        if correlated == 0 {
            return Err(
                super::super::UiRebindPlanningDenial::ProjectionSchemaTransitionUncorrelated {
                    component_identity: edge.component_identity().into(),
                },
            );
        }
    }
    Ok(governed)
}

pub(super) fn classify_transition(
    predecessor: &UiProjectionSchemaRequirement,
    candidate: &UiProjectionSchemaRequirement,
    installed: &UiProjectionSchemaRequirement,
) -> Option<UiProjectionSchemaTransitionKind> {
    match (predecessor == installed, candidate == installed) {
        (_, false) => Some(UiProjectionSchemaTransitionKind::Stopped),
        (false, true) => Some(UiProjectionSchemaTransitionKind::Recovered),
        (true, true) => None,
    }
}

fn schema_requirement(
    authored: &crate::runtime::WorthUiAuthoredProjectionRequirement,
) -> Option<UiProjectionSchemaRequirement> {
    authored
        .scalar_requirement()
        .cloned()
        .map(UiProjectionSchemaRequirement::Scalar)
        .or_else(|| {
            authored
                .collection_requirement()
                .cloned()
                .map(UiProjectionSchemaRequirement::Collection)
        })
}

fn installed_requirement(
    plan: &worth_ui_query_binding::WorthUiQueryBindingPlan,
    identity: &worth_ui_query_binding::WorthUiQueryViewIdentity,
) -> Option<UiProjectionSchemaRequirement> {
    plan.scalar_projection_registration(identity)
        .map(|registration| {
            UiProjectionSchemaRequirement::Scalar(registration.requirement().clone())
        })
        .or_else(|| {
            plan.application_scalar_projection_registration(identity)
                .map(|registration| {
                    UiProjectionSchemaRequirement::Scalar(registration.requirement().clone())
                })
        })
        .or_else(|| {
            plan.collection_projection_registration(identity)
                .map(|registration| {
                    UiProjectionSchemaRequirement::Collection(registration.requirement().clone())
                })
        })
}

fn insert_content(
    content: &mut crate::mounting::UiMountedSemanticContentInput,
    graph_node: crate::graph::UiGraphNodeIdentity,
    installed: &UiProjectionSchemaRequirement,
    kind: UiProjectionSchemaTransitionKind,
) -> Result<(), super::super::UiRebindPlanningDenial> {
    let posture = Arc::from(match kind {
        UiProjectionSchemaTransitionKind::Stopped => "SCHEMA MISMATCH",
        UiProjectionSchemaTransitionKind::Recovered => "CURRENT",
    });
    let inserted = match installed {
        UiProjectionSchemaRequirement::Scalar(_) => content.insert_scalar(
            graph_node,
            crate::mounting::UiMountedSemanticTextValueDirective::Preserve,
            posture,
        ),
        UiProjectionSchemaRequirement::Collection(_) => content.insert_collection(
            graph_node,
            crate::mounting::UiMountedCollectionTextDirective::Preserve,
            posture,
        ),
    };
    inserted.map_err(
        |()| super::super::UiRebindPlanningDenial::AmbiguousProjectionContent { graph_node },
    )
}
