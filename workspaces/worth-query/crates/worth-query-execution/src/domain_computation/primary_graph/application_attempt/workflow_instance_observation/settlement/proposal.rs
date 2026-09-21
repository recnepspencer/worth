use worth_foundational::facade::{AspectValue, InternedString};

use super::super::ObservedWorkflowTransition;
use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_latest_workflow_proposal_identity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transitions: &[ObservedWorkflowTransition],
    maximum_facts: usize,
) -> Result<(String, Vec<WorthQueryApplicationObservedFact>), WorthQueryApplicationAttemptDenial> {
    if transitions.len() > maximum_facts {
        return Err(budget_denial());
    }
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let mut facts = Vec::new();
    let mut latest = None;
    for transition in transitions {
        let relations = observe_adjacency(
            runtime,
            snapshot,
            layout.transition_proposal_relation,
            transition.entity,
            direction,
            2,
        )
        .ok_or_else(|| denial("workflow proposal relation is unavailable"))?;
        facts.push(WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.transition_proposal_relation,
            anchor: transition.entity,
            direction,
            maximum_work_units: 2,
            relations: relations.clone(),
        });
        match relations.as_slice() {
            [] => {}
            [relation] => {
                if facts.len().saturating_add(2) > maximum_facts {
                    return Err(budget_denial());
                }
                let proposal = relation.to;
                let kind = layout.proposal.entity_kind;
                let value = observe_field_value(
                    runtime,
                    snapshot,
                    proposal,
                    kind,
                    &layout.proposal.identity,
                )
                .ok_or_else(|| denial("workflow proposal identity is unavailable"))?;
                let AspectValue::String(InternedString::Raw(identity)) = &value else {
                    return Err(denial("workflow proposal identity has the wrong type"));
                };
                if decode_identity(identity).is_none() {
                    return Err(denial("workflow proposal identity is malformed"));
                }
                let identity = identity.clone();
                facts.push(WorthQueryApplicationObservedFact::Entity {
                    entity_id: proposal,
                    kind,
                });
                facts.push(WorthQueryApplicationObservedFact::Field {
                    entity_id: proposal,
                    kind,
                    locator: layout.proposal.identity.clone(),
                    value,
                });
                if latest
                    .as_ref()
                    .is_none_or(|(occurrence, _)| transition.settlement.occurrence() > *occurrence)
                {
                    latest = Some((transition.settlement.occurrence(), identity));
                }
            }
            _ => return Err(denial("workflow transition has duplicate proposals")),
        }
        if facts.len() > maximum_facts {
            return Err(budget_denial());
        }
    }
    latest
        .map(|(_, identity)| (identity, facts))
        .ok_or_else(|| denial("workflow assessment has no proposal dependency"))
}

fn budget_denial() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
        "workflow proposal binding fact budget",
    )
}

fn decode_identity(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut identity = [0_u8; 32];
    for (index, byte) in identity.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(identity)
}

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
