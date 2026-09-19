use super::*;
use crate::data::checkpoint::CheckpointBarrier;
use crate::diagnostics::epochs::EventEpochOutcome;
use crate::diagnostics::state::DiagnosticsState;
use crate::facade::{EvaluationContext, EvaluationRequestMode, SignalGraph};
use crate::tests::support::version_ab;

pub(super) fn recorded_flow() -> FlowSummary {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(7, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    let mut flow = graph
        .observe()
        .latest_flow_diagnostics()
        .unwrap()
        .to_owned_summary();
    // Amplify retained detail for this storage proof; it is not execution evidence.
    flow.cause_samples = (0..256)
        .map(|_| FlowCauseSample {
            node,
            cause_kinds: vec!["retained cause detail".repeat(64)],
            scope_kinds: Vec::new(),
            scope_notes: Vec::new(),
            suspect_classes: Vec::new(),
            rewired: false,
            conservative_recompute: false,
        })
        .collect();
    flow
}

fn epoch(ordinal: u32) -> EventEpochSummary {
    EventEpochSummary {
        ordinal,
        barrier: CheckpointBarrier::PerOperation,
        emitted_event_count: 0,
        subscriber_count: 0,
        committed_subscriber_count: 0,
        failed_subscriber_position: None,
        subscriber_outcomes: Vec::new(),
        outcome: EventEpochOutcome::Committed,
        failure_subscriber: None,
        message: Some(format!("epoch {ordinal}")),
    }
}

#[test]
fn recorder_annotation_updates_share_payload_and_preserve_sibling_annotations() {
    let mut original = recorded_flow();
    original.event_epochs = vec![epoch(1)];
    original.observation = Some(ObservationBoundarySummary::default());
    let mut source = DiagnosticsState::default();
    source.latest_flow = Some(original.clone().into());
    let mut draft = source.clone();
    let source_flow = source.latest_flow.as_ref().unwrap();
    let draft_flow = draft.latest_flow.as_ref().unwrap();
    assert!(Arc::ptr_eq(&source_flow.payload, &draft_flow.payload));
    assert!(Arc::ptr_eq(
        &source_flow.event_epochs,
        &draft_flow.event_epochs
    ));
    assert!(Arc::ptr_eq(
        source_flow.observation.as_ref().unwrap(),
        draft_flow.observation.as_ref().unwrap()
    ));
    let observation = ObservationBoundarySummary {
        classified_event_count: 2,
        ..Default::default()
    };
    draft.record_observation(observation.clone());
    draft.attach_event_epochs_to_latest_flow(vec![epoch(2)]);
    let draft_flow = draft.latest_flow.as_ref().unwrap();
    assert!(Arc::ptr_eq(&source_flow.payload, &draft_flow.payload));
    assert!(!Arc::ptr_eq(
        &source_flow.event_epochs,
        &draft_flow.event_epochs
    ));
    assert_eq!(source_flow.view().to_owned_summary(), original);
    assert_eq!(draft_flow.view().observation, Some(&observation));
    assert_eq!(draft_flow.view().event_epochs, &[epoch(2)]);
    assert_eq!(draft.latest_observation(), Some(&observation));
    assert!(Arc::ptr_eq(
        draft_flow.observation.as_ref().unwrap(),
        draft.latest_observation.as_ref().unwrap()
    ));
}

#[test]
fn retained_flow_serialization_matches_owned_wire_and_preserves_legacy_defaults() {
    let mut original = recorded_flow();
    original.event_epochs = vec![epoch(3)];
    original.observation = Some(ObservationBoundarySummary {
        classified_event_count: 3,
        ..Default::default()
    });
    let retained = RetainedFlow::from(original.clone());
    let wire = serde_json::to_string(&original).unwrap();
    assert_eq!(serde_json::to_string(&retained).unwrap(), wire);
    assert_eq!(serde_json::to_string(&retained.view()).unwrap(), wire);
    let restored: RetainedFlow = serde_json::from_str(&wire).unwrap();
    assert_eq!(restored.view().to_owned_summary(), original);
    let mut legacy = serde_json::to_value(&original).unwrap();
    for key in ["event_epochs", "observation", "cause_samples"] {
        legacy.as_object_mut().unwrap().remove(key);
    }
    let expected: FlowSummary = serde_json::from_value(legacy.clone()).unwrap();
    let restored: RetainedFlow = serde_json::from_value(legacy).unwrap();
    assert_eq!(restored.view().to_owned_summary(), expected);
}
