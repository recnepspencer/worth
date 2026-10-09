use crate::data::checkpoint::CheckpointBarrier;
use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
use crate::data::subscriber_context::SubscriberContext;
use crate::facade::{SignalError, SignalGraph, SignalRuntime, SignalRuntimePolicy};
use crate::tests::support::ASPECT_A;

use super::workflow_truth::FailureInjectionPoint;

type EventRuntime = SignalRuntime<EventDomain, (), WorkflowEvent, (), ()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EventDomain {
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowEvent {
    Tick,
}

struct FailingFlushSubscriber;

impl EventSubscriber for FailingFlushSubscriber {
    type Event = WorkflowEvent;
    type DataId = EventDomain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(901)
    }

    fn name(&self) -> &'static str {
        "adversarial-flush-failure"
    }

    fn requires(&self) -> &'static [Self::DataId] {
        &[EventDomain::Audit]
    }

    fn provides(&self) -> &'static [Self::DataId] {
        &[EventDomain::Audit]
    }

    fn on_event(&mut self, _event: &Self::Event) {}

    fn on_checkpoint(
        &mut self,
        _barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        Err(SignalError::internal("synthetic flush failure"))
    }
}

#[test]
fn event_flush_failure_workflow_does_not_advance_branch_truth() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    assert!(matches!(
        FailureInjectionPoint::DuringEventFlush,
        FailureInjectionPoint::DuringEventFlush
    ));
    let mut runtime: EventRuntime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_domains::<EventDomain>()
        .with_events::<WorkflowEvent>()
        .runtime_policy(SignalRuntimePolicy::development().with_history_limit(4))
        .build();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailingFlushSubscriber))
        .unwrap();

    let source = runtime.graph_mut().node().build();
    let baseline = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let feature = runtime.create_branch("event-feature").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let head_before = runtime.observe().branch_head_snapshot_id(feature.id);
    let replay_before = runtime.observe().replay_for_branch(feature.id);

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(source, ASPECT_A).unwrap();
    tx.emit_event(WorkflowEvent::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();
    let outcome = tx.commit();
    assert!(outcome.is_err());

    assert_eq!(
        runtime.observe().branch_head_snapshot_id(feature.id),
        head_before
    );
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        0
    );
    let replay_after = runtime.observe().replay_for_branch(feature.id);
    assert!(
        replay_after.frames.len() >= replay_before.frames.len(),
        "flush failure should be visible without silently advancing branch truth"
    );
    runtime
        .switch_branch(
            runtime
                .branch_ancestry(feature.id)
                .first()
                .cloned()
                .unwrap(),
        )
        .unwrap();
    runtime
        .restore_branch_snapshot(runtime.observe().current_branch(), &baseline)
        .unwrap();
}
