//! A cancellation that already committed under this key replays from the
//! instance's recorded history. It reads no definition, so the replay holds
//! after the branch adopts a program that did not carry the instance's.

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::super::workflow_instance_observation::observe_ended_history;
use super::super::super::{
    observe_field_value, WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use super::super::performed;
use super::HISTORY;
use crate::domain_computation::primary_graph::workflow::{
    instance::{WorkflowInstanceState, WorkflowPerformedEffect},
    schema::WorthQueryWorkflowLayout,
};

/// What the recorded cancellation reports, or `None` when the instance
/// records no cancellation under `identity`.
pub(super) fn observe_recorded_cancellation(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    identity: &str,
    maximum_transitions: usize,
    budget: WorthQueryWorkflowHistoryReconstructionBudget,
) -> Result<
    Option<(
        Vec<WorkflowPerformedEffect>,
        Vec<WorthQueryApplicationObservedFact>,
    )>,
    WorthQueryApplicationAttemptDenial,
> {
    let recorded = observe_field_value(
        runtime,
        snapshot,
        instance,
        layout.instance.entity_kind,
        &layout.instance.cancellation_identity,
    );
    if recorded
        != Some(AspectValue::String(InternedString::Raw(
            identity.to_owned(),
        )))
    {
        return Ok(None);
    }
    // A cancelled instance never becomes ready again, so this false fact
    // makes the program resolve the exact replay; it never commits empty.
    let mut facts = vec![WorthQueryApplicationObservedFact::Field {
        entity_id: instance,
        kind: layout.instance.entity_kind,
        locator: layout.instance.state.clone(),
        value: AspectValue::UInt64(WorkflowInstanceState::Ready.persisted_tag()),
    }];
    let (mut performed, history_facts) = observe_ended_history(
        runtime,
        snapshot,
        layout,
        instance,
        maximum_transitions,
        budget,
    )?;
    facts.extend(history_facts);
    performed.extend(performed::inherited_effects(
        runtime,
        snapshot,
        layout,
        instance,
        maximum_transitions,
        &mut facts,
        HISTORY,
    )?);
    Ok(Some((performed, facts)))
}
