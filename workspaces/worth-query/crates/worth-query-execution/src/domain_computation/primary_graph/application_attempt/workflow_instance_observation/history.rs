use super::{
    denial, observe_adjacency, settlement, ObservedWorkflowTransition,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::CompiledWorkflowDefinition, schema::WorthQueryWorkflowLayout,
};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget;
use worth_relational::facade::identity::EntityId;

mod budget;
use super::super::fact::{observe_adjacency_checked, AdjacencyObservationDenial};
use budget::HistoryReconstructionCharge;

/// What the history basis a read records must keep true at commit.
#[derive(Clone, Copy)]
pub(super) enum HistoryBasis {
    /// The request writes one more transition, so a slot must remain free
    /// once the steps the instance inherited from its sources are spent.
    Advance { inherited_steps: usize },
    /// The request ends the live instance and writes no transition, so a
    /// history that fills the whole retained capacity still closes.
    Close,
    /// The instance has already ended; the read only reports its history.
    Ended,
}

pub(super) struct ObservedWorkflowHistory {
    pub(super) transitions: Vec<ObservedWorkflowTransition>,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
    pub(super) transition_visits: usize,
    pub(super) charge_bytes: usize,
}

/// The settled history of an instance that has already ended, read only to
/// report what it performed. It admits no progression.
pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_ended_history(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    maximum_transitions: usize,
    compiled: &CompiledWorkflowDefinition,
    history_budget: WorthQueryWorkflowHistoryReconstructionBudget,
) -> Result<
    (
        Vec<ObservedWorkflowTransition>,
        Vec<WorthQueryApplicationObservedFact>,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let history = observe(
        runtime,
        snapshot,
        layout,
        instance,
        HistoryBasis::Ended,
        maximum_transitions,
        compiled,
        history_budget,
    )?;
    Ok((history.transitions, history.facts))
}

pub(super) fn observe(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    basis: HistoryBasis,
    maximum_transitions: usize,
    compiled: &CompiledWorkflowDefinition,
    budget: WorthQueryWorkflowHistoryReconstructionBudget,
) -> Result<ObservedWorkflowHistory, WorthQueryApplicationAttemptDenial> {
    let mut charge = HistoryReconstructionCharge::new(budget)?;
    let inventory_limit = charge.inventory_limit(maximum_transitions);
    let transitions = observe_adjacency_checked(
        runtime,
        snapshot,
        layout.instance_transition_relation,
        instance,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        inventory_limit.saturating_mul(2),
    )
    .map_err(|failure| match failure {
        AdjacencyObservationDenial::SnapshotUnavailable => {
            denial("workflow instance transition inventory is unavailable")
        }
        AdjacencyObservationDenial::WorkBudgetExceeded => budget::inventory_exhausted(),
    })?;
    let transition_count = transitions.len();
    charge.reserve_inventory(transition_count)?;
    let mut settled = Vec::with_capacity(transition_count);
    let mut scratch = Vec::new();
    for transition in transitions {
        charge.reserve_record(
            runtime,
            snapshot,
            transition.to,
            layout.transition.entity_kind,
            &layout.transition.protocol_version,
        )?;
        let evidence = observe_adjacency(
            runtime,
            snapshot,
            layout.transition_assessment_evidence_relation,
            transition.to,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            2,
        )
        .ok_or_else(|| denial("workflow assessment evidence relation is unavailable"))?;
        for evidence in evidence {
            charge.reserve_record(
                runtime,
                snapshot,
                evidence.to,
                layout.assessment_evidence.entity_kind,
                &layout.assessment_evidence.protocol_version,
            )?;
        }
        scratch.clear();
        let settlement = settlement::observe_settled_transition(
            runtime,
            snapshot,
            layout,
            transition.to,
            &mut scratch,
        )?;
        let identity = match super::observe_field_value(
            runtime,
            snapshot,
            transition.to,
            layout.transition.entity_kind,
            &layout.transition.identity,
        ) {
            Some(AspectValue::String(InternedString::Raw(identity))) => identity,
            _ => return Err(denial("workflow transition identity is unavailable")),
        };
        let node = compiled
            .node(settlement.node())
            .ok_or_else(|| denial("history node is absent from the compiled definition"))?;
        charge.reserve_path(node.path())?;
        let assessment_evidence = settlement::observe_assessment_evidence(
            runtime,
            snapshot,
            layout,
            transition.to,
            &mut scratch,
        )?;
        settled.push(ObservedWorkflowTransition {
            entity: transition.to,
            identity,
            settlement,
            assessment_evidence,
        });
    }
    // Every read above used this immutable native basis. A compact exact-basis
    // comparison protects all those reads; scratch observations are not ordinary
    // decision facts and cannot confer mutation scope.
    Ok(ObservedWorkflowHistory {
        transitions: settled,
        facts: vec![WorthQueryApplicationObservedFact::WorkflowHistoryBasis {
            instance,
            maximum_transitions: match basis {
                HistoryBasis::Advance { inherited_steps } => {
                    maximum_transitions.saturating_sub(inherited_steps)
                }
                HistoryBasis::Close => transition_count.saturating_add(1),
                HistoryBasis::Ended => transition_count,
            },
            transition_count,
            snapshot: snapshot.clone(),
        }],
        transition_visits: transition_count,
        charge_bytes: charge.bytes(),
    })
}
