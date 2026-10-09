use std::sync::{Arc, Mutex};

use crate::data::checkpoint::CheckpointBarrier;
use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
use crate::data::subscriber_context::SubscriberContext;
use crate::facade::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Domain {
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ev {
    Tick(u32),
}

#[derive(Clone)]
struct EpochRecorder {
    seen: Arc<Mutex<Vec<u32>>>,
    flushes: Arc<Mutex<Vec<(CheckpointBarrier, Vec<u32>)>>>,
}

impl EventSubscriber for EpochRecorder {
    type Event = Ev;
    type DataId = Domain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(1)
    }
    fn name(&self) -> &'static str {
        "epoch-recorder"
    }
    fn requires(&self) -> &'static [Self::DataId] {
        &[]
    }
    fn provides(&self) -> &'static [Self::DataId] {
        &[]
    }
    fn on_event(&mut self, event: &Self::Event) {
        let Ev::Tick(value) = event;
        self.seen.lock().unwrap().push(*value);
    }
    fn on_checkpoint(
        &mut self,
        barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        let mut seen = self.seen.lock().unwrap();
        self.flushes
            .lock()
            .unwrap()
            .push((barrier, std::mem::take(&mut *seen)));
        Ok(())
    }
}

#[derive(Clone)]
struct EpochContextWriter {
    seen: Arc<Mutex<Vec<u32>>>,
    staged: Option<u32>,
}

impl EventSubscriber for EpochContextWriter {
    type Event = Ev;
    type DataId = Domain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(2)
    }

    fn name(&self) -> &'static str {
        "epoch-context-writer"
    }

    fn requires(&self) -> &'static [Self::DataId] {
        &[]
    }

    fn provides(&self) -> &'static [Self::DataId] {
        &[Domain::Audit]
    }

    fn on_begin(
        &mut self,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) {
        self.staged = None;
    }

    fn on_event(&mut self, event: &Self::Event) {
        let Ev::Tick(value) = event;
        self.seen.lock().unwrap().push(*value);
        self.staged = Some(*value);
    }

    fn on_checkpoint(
        &mut self,
        _barrier: CheckpointBarrier,
        ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        ctx.stage(Domain::Audit, self.staged.unwrap_or_default())
            .map_err(|err| SignalError::internal(format!("{err:?}")))
    }
}

#[derive(Clone)]
struct BarrierFailGate {
    fail_on: Arc<Mutex<Option<CheckpointBarrier>>>,
}

impl EventSubscriber for BarrierFailGate {
    type Event = Ev;
    type DataId = Domain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(3)
    }

    fn name(&self) -> &'static str {
        "barrier-fail-gate"
    }

    fn requires(&self) -> &'static [Self::DataId] {
        &[Domain::Audit]
    }

    fn provides(&self) -> &'static [Self::DataId] {
        &[]
    }

    fn on_event(&mut self, _event: &Self::Event) {}

    fn on_checkpoint(
        &mut self,
        barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        if self.fail_on.lock().unwrap().as_ref() == Some(&barrier) {
            Err(SignalError::internal("epoch failure injection"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn transaction_flushes_deliver_events_in_epoch_order() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .build();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let flushes = Arc::new(Mutex::new(Vec::new()));
    runtime
        .event_bus_mut()
        .subscribe(Box::new(EpochRecorder {
            seen: seen.clone(),
            flushes: flushes.clone(),
        }))
        .unwrap();

    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            tx.emit_event(Ev::Tick(1));
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            tx.emit_event(Ev::Tick(2));
            tx.flush_events(CheckpointBarrier::PerCommit)?;
            Ok(())
        })
        .unwrap();

    let flushes = flushes.lock().unwrap().clone();
    assert_eq!(flushes.len(), 2);
    assert_eq!(flushes[0].1, vec![1]);
    assert_eq!(flushes[1].1, vec![2]);
}

#[test]
fn failed_later_epoch_keeps_earlier_epoch_committed_and_does_not_replay_stale_events() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .build();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let fail_on = Arc::new(Mutex::new(None));
    runtime
        .event_bus_mut()
        .subscribe(Box::new(EpochContextWriter {
            seen: seen.clone(),
            staged: None,
        }))
        .unwrap();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(BarrierFailGate {
            fail_on: fail_on.clone(),
        }))
        .unwrap();

    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            tx.emit_event(Ev::Tick(1));
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        runtime
            .event_bus()
            .context()
            .committed::<u32>(Domain::Audit),
        Some(&1)
    );

    *fail_on.lock().unwrap() = Some(CheckpointBarrier::PerCommit);
    let err = runtime
        .transaction(request_execution, &mut ctx, |tx| {
            tx.emit_event(Ev::Tick(7));
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            tx.emit_event(Ev::Tick(11));
            tx.flush_events(CheckpointBarrier::PerCommit)?;
            Ok(())
        })
        .unwrap_err();
    assert!(format!("{err}").contains("event bus flush failed"));
    assert_eq!(
        runtime
            .event_bus()
            .context()
            .committed::<u32>(Domain::Audit),
        Some(&7),
        "earlier successful epoch should stay committed when a later epoch fails"
    );
    assert!(
        runtime
            .event_bus()
            .context()
            .staged::<u32>(Domain::Audit)
            .is_none(),
        "failed later epoch must not leave staged subscriber context behind"
    );
    let failure = runtime.observe().latest_failure_diagnostics().unwrap();
    assert_eq!(failure.event_epochs.len(), 2);
    assert_eq!(failure.event_epochs[0].committed_subscriber_count, 2);
    assert_eq!(failure.event_epochs[1].committed_subscriber_count, 1);
    assert_eq!(failure.event_epochs[1].failed_subscriber_position, Some(2));
    assert!(failure.event_epochs[0]
        .subscriber_outcomes
        .iter()
        .any(|outcome| {
            outcome.subscriber_name == "epoch-context-writer"
                && outcome.provides_data_ids == vec!["Audit".to_string()]
                && outcome.staged_data_ids == vec!["Audit".to_string()]
        }));
    assert!(failure.event_epochs[0]
        .subscriber_outcomes
        .iter()
        .all(|outcome| outcome.outcome == EventSubscriberOutcomeKind::Committed));
    assert!(failure.event_epochs[1]
        .subscriber_outcomes
        .iter()
        .any(|outcome| outcome.outcome == EventSubscriberOutcomeKind::Failed));
    assert!(failure.event_epochs[1]
        .subscriber_outcomes
        .iter()
        .any(|outcome| {
            outcome.subscriber_name == "barrier-fail-gate"
                && outcome.requires_data_ids == vec!["Audit".to_string()]
                && outcome.provides_data_ids.is_empty()
                && outcome.outcome == EventSubscriberOutcomeKind::Failed
        }));

    *fail_on.lock().unwrap() = None;
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            tx.emit_event(Ev::Tick(13));
            tx.flush_events(CheckpointBarrier::PerCommit)?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        runtime
            .event_bus()
            .context()
            .committed::<u32>(Domain::Audit),
        Some(&13)
    );
    assert_eq!(
        &*seen.lock().unwrap(),
        &[1, 7, 11, 13],
        "failed later epoch events must not replay on the next successful transaction"
    );
}
