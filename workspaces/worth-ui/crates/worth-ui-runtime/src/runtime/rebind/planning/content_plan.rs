use std::sync::Arc;

use crate::graph::UiGraphFactConsumerIdentity;

mod collection;
mod schema_transition;

pub(super) fn compile_content_plan(
    predecessor: &crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationAuthority,
    candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    scope: &super::super::UiResolvedAffectedScope,
) -> Result<crate::mounting::UiMountedSemanticContentInput, super::UiRebindPlanningDenial> {
    let mut content = crate::mounting::UiMountedSemanticContentInput::empty();
    let projection_input_capacity = candidate.query_binding_plan().projection_input_count();
    if predecessor.query_binding_plan() == candidate.query_binding_plan() {
        content.merge_projection_inputs(projection_input_capacity);
    } else {
        content.replace_projection_inputs(projection_input_capacity);
    }
    retain_projection_inputs(candidate, scope, &mut content)?;
    let governed_nodes = schema_transition::compile(predecessor, candidate, scope, &mut content)?;
    project_intent_postures(candidate, scope, &governed_nodes, &mut content)?;
    for lookup in scope.lookups() {
        project_query_content(candidate, scope, lookup, &governed_nodes, &mut content)?;
    }
    Ok(content)
}

fn project_query_content(
    candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    scope: &super::super::UiResolvedAffectedScope,
    lookup: &super::super::UiAffectedFactLookup,
    governed: &std::collections::BTreeSet<crate::graph::UiGraphNodeIdentity>,
    content: &mut crate::mounting::UiMountedSemanticContentInput,
) -> Result<(), super::UiRebindPlanningDenial> {
    let Some(query) = scope
        .facts()
        .get(lookup.fact_ordinal())
        .and_then(crate::fact_contract::UiProducedFact::query)
    else {
        return Ok(());
    };
    let Some((identity, projected)) = projected_query_content(query)? else {
        return Ok(());
    };
    for entry in lookup.candidate().entries().iter().filter(|entry| {
        candidate
            .semantic_handoff()
            .projection_contents()
            .iter()
            .any(|edge| {
                edge.projection_identity() == identity
                    && edge.component_identity() == entry.consumer_key().authored_identity()
            })
    }) {
        let UiGraphFactConsumerIdentity::GraphNode(graph_node) = entry.consumer() else {
            continue;
        };
        if !governed.contains(&graph_node) {
            insert_projected_content(content, graph_node, &projected)?;
        }
    }
    Ok(())
}

fn projected_query_content(
    query: &crate::fact_contract::UiQueryChangedFact,
) -> Result<
    Option<(
        &worth_ui_query_binding::WorthUiQueryViewIdentity,
        UiProjectedSemanticContent,
    )>,
    super::UiRebindPlanningDenial,
> {
    Ok(
        match (
            query.scalar_projection(),
            query.application_scalar_projection(),
            query.collection_projection(),
        ) {
            (Some(scalar), None, None) => Some((
                scalar.core().projection_identity(),
                UiProjectedSemanticContent::Scalar(project_scalar(scalar)),
            )),
            (None, Some(scalar), None) => Some((
                scalar.projection_identity(),
                UiProjectedSemanticContent::Scalar(project_application_scalar(scalar)),
            )),
            (None, None, Some(collection)) => Some((
                collection.core().projection_identity(),
                UiProjectedSemanticContent::Collection(collection::project_collection(collection)?),
            )),
            (None, None, None) => None,
            _ => unreachable!("a Query projection fact has one sealed shape"),
        },
    )
}

fn insert_projected_content(
    content: &mut crate::mounting::UiMountedSemanticContentInput,
    graph_node: crate::graph::UiGraphNodeIdentity,
    projected: &UiProjectedSemanticContent,
) -> Result<(), super::UiRebindPlanningDenial> {
    let inserted = match projected {
        UiProjectedSemanticContent::Scalar((value, posture)) => {
            content.insert_scalar(graph_node, value.clone(), Arc::clone(posture))
        }
        UiProjectedSemanticContent::Collection((value, posture)) => {
            content.insert_collection(graph_node, value.clone(), Arc::clone(posture))
        }
    };
    inserted.map_err(|()| super::UiRebindPlanningDenial::AmbiguousProjectionContent { graph_node })
}

fn project_intent_postures(
    candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    scope: &super::super::UiResolvedAffectedScope,
    governed_nodes: &std::collections::BTreeSet<crate::graph::UiGraphNodeIdentity>,
    content: &mut crate::mounting::UiMountedSemanticContentInput,
) -> Result<(), super::UiRebindPlanningDenial> {
    for lookup in scope.lookups() {
        let Some(posture) = scope
            .facts()
            .get(lookup.fact_ordinal())
            .and_then(crate::fact_contract::UiProducedFact::intent_posture)
        else {
            continue;
        };
        for entry in lookup.candidate().entries() {
            let UiGraphFactConsumerIdentity::GraphNode(graph_node) = entry.consumer() else {
                continue;
            };
            if graph_node != posture.graph_node() || governed_nodes.contains(&graph_node) {
                continue;
            }
            // Posture observation does not itself grant text presentation. A
            // component must have admitted semantic text before receiving the
            // existing textual posture projection; appearance-only owners keep
            // their separately authored paint and product-owned copy.
            let snapshot = candidate.graph_snapshot();
            let renders_text = snapshot
                .core_indexes()
                .node_identity()
                .node(snapshot.nodes(), graph_node)
                .and_then(|node| {
                    crate::graph::component_capability_for_node(
                        node,
                        candidate.capabilities(),
                        &candidate.authored_declaration_lookup(),
                    )
                })
                .is_some_and(|component| component.semantic_text_contract().is_some());
            if !renders_text {
                continue;
            }
            let label: Arc<str> = match posture.posture() {
                crate::fact_contract::UiIntentPostureKind::Admitted => Arc::from("ADMITTED"),
                crate::fact_contract::UiIntentPostureKind::ConfirmationRequired => {
                    Arc::from("CONFIRMATION REQUIRED")
                }
                crate::fact_contract::UiIntentPostureKind::Completed => Arc::from("COMPLETED"),
                crate::fact_contract::UiIntentPostureKind::Denied => Arc::from("DENIED"),
                crate::fact_contract::UiIntentPostureKind::StaleConfirmation => {
                    Arc::from("STALE CONFIRMATION")
                }
                crate::fact_contract::UiIntentPostureKind::Cancelled => Arc::from("CANCELLED"),
            };
            content
                .insert_scalar(
                    graph_node,
                    crate::mounting::UiMountedSemanticTextValueDirective::Preserve,
                    label,
                )
                .map_err(
                    |()| super::UiRebindPlanningDenial::AmbiguousProjectionContent { graph_node },
                )?;
        }
    }
    Ok(())
}

fn retain_projection_inputs(
    candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    scope: &super::super::UiResolvedAffectedScope,
    content: &mut crate::mounting::UiMountedSemanticContentInput,
) -> Result<(), super::UiRebindPlanningDenial> {
    for fact in scope.facts() {
        let Some(query) = fact.query() else {
            continue;
        };
        let input = match (
            query.scalar_projection(),
            query.application_scalar_projection(),
            query.collection_projection(),
        ) {
            (Some(scalar), None, None) => {
                projection_input(candidate, scalar.core().projection_identity(), |slot| {
                    scalar.intent_input_transition(slot)
                })?
            }
            (None, Some(scalar), None) => {
                projection_input(candidate, scalar.projection_identity(), |slot| {
                    scalar.intent_input_transition(slot)
                })?
            }
            (None, None, Some(collection)) => {
                projection_input(candidate, collection.core().projection_identity(), |slot| {
                    collection.intent_input_transition(slot)
                })?
            }
            (None, None, None) => continue,
            _ => unreachable!("a Query projection fact has one sealed shape"),
        };
        let projection = input.revision().projection_identity().clone();
        content
            .insert_projection_input_transition(input)
            .map_err(|()| super::UiRebindPlanningDenial::AmbiguousProjectionInput { projection })?;
    }
    Ok(())
}

fn projection_input(
    candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    projection: &worth_ui_query_binding::WorthUiQueryViewIdentity,
    materialize: impl FnOnce(
        worth_ui_query_binding::UiProjectionInputSlot,
    ) -> worth_ui_query_binding::UiProjectionInputFactTransition,
) -> Result<worth_ui_query_binding::UiProjectionInputFactTransition, super::UiRebindPlanningDenial>
{
    let slot = candidate
        .query_binding_plan()
        .projection_input_slot(projection)
        .ok_or_else(
            || super::UiRebindPlanningDenial::ProjectionInputSlotUnavailable {
                projection: projection.clone(),
            },
        )?;
    Ok(materialize(slot))
}

enum UiProjectedSemanticContent {
    Scalar(
        (
            crate::mounting::UiMountedSemanticTextValueDirective,
            Arc<str>,
        ),
    ),
    Collection((crate::mounting::UiMountedCollectionTextDirective, Arc<str>)),
}

fn project_scalar(
    fact: &worth_ui_query_binding::UiScalarProjectionFactReceipt,
) -> (
    crate::mounting::UiMountedSemanticTextValueDirective,
    Arc<str>,
) {
    use worth_ui_query_binding::{UiPresentProjection, UiProjectionAvailability};

    match fact.availability() {
        UiProjectionAvailability::Unavailable(receipt) => (
            crate::mounting::UiMountedSemanticTextValueDirective::Clear,
            unavailable_label(receipt.kind()),
        ),
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => (
            crate::mounting::UiMountedSemanticTextValueDirective::Replace(Arc::from(
                value.as_str(),
            )),
            Arc::from("CURRENT"),
        ),
        UiProjectionAvailability::Present(UiPresentProjection::RetainedStale {
            value,
            activity,
        }) => (
            crate::mounting::UiMountedSemanticTextValueDirective::Replace(Arc::from(
                value.as_str(),
            )),
            retained_label(activity.kind()),
        ),
        UiProjectionAvailability::Stopped(receipt) => (
            crate::mounting::UiMountedSemanticTextValueDirective::Preserve,
            stopped_label(receipt.kind()),
        ),
    }
}

fn project_application_scalar(
    fact: &worth_ui_query_binding::UiApplicationScalarProjectionFactReceipt,
) -> (
    crate::mounting::UiMountedSemanticTextValueDirective,
    Arc<str>,
) {
    if fact.value().revision == 0 {
        (
            crate::mounting::UiMountedSemanticTextValueDirective::Clear,
            Arc::from("PENDING"),
        )
    } else {
        (
            crate::mounting::UiMountedSemanticTextValueDirective::Replace(Arc::from(
                fact.value().status.as_str(),
            )),
            Arc::from("CURRENT"),
        )
    }
}

fn unavailable_label(kind: worth_ui_query_binding::UiProjectionUnavailableKind) -> Arc<str> {
    use worth_ui_query_binding::UiProjectionUnavailableKind as Kind;
    Arc::from(match kind {
        Kind::Unresolved => "UNRESOLVED",
        Kind::Pending => "PENDING",
        Kind::Failed => "FAILED",
        Kind::Cancelled => "CANCELLED",
        Kind::Retried => "RETRIED",
        Kind::Superseded => "SUPERSEDED",
        Kind::Denied => "DENIED",
        Kind::Unsupported => "UNSUPPORTED",
        Kind::Remasked => "REMASKED",
        Kind::BasisDrift => "BASIS DRIFT",
        Kind::GenerationDrift => "GENERATION DRIFT",
    })
}

fn retained_label(kind: worth_ui_query_binding::UiProjectionRetainedActivityKind) -> Arc<str> {
    use worth_ui_query_binding::UiProjectionRetainedActivityKind as Kind;
    Arc::from(match kind {
        Kind::Idle => "STALE",
        Kind::Revalidating => "REVALIDATING",
    })
}

fn stopped_label(kind: worth_ui_query_binding::UiProjectionFactStopKind) -> Arc<str> {
    use worth_ui_query_binding::UiProjectionFactStopKind as Kind;
    Arc::from(match kind {
        Kind::SchemaMismatch => "SCHEMA MISMATCH",
        Kind::PayloadShapeMismatch => "PAYLOAD SHAPE MISMATCH",
        Kind::NativeFamilyMismatch => "NATIVE FAMILY MISMATCH",
        Kind::WrongWorld => "WRONG WORLD",
        Kind::StaleBindingGeneration => "STALE BINDING",
        Kind::StaleResultGeneration => "STALE RESULT",
        Kind::BasisMismatch => "BASIS MISMATCH",
        Kind::Unsupported => "UNSUPPORTED",
        Kind::Remasked => "REMASKED",
        Kind::BudgetExceeded => "BUDGET EXCEEDED",
        Kind::ResetRequired => "RESET REQUIRED",
    })
}

#[cfg(test)]
#[path = "content_plan/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "content_plan/schema_transition_tests.rs"]
mod schema_transition_tests;
