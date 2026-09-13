use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::data::checkpoint::CheckpointBarrier;
use crate::data::error::SignalError;
use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
use crate::data::subscriber_context::SubscriberContext;
use crate::diagnostics::replay::ReplayEventKind;
use crate::diagnostics::ExecutionFailurePhase;
use crate::facade::*;
use crate::tests::support::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Domain {
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ev {
    Tick,
}

struct FailingSubscriber;

impl EventSubscriber for FailingSubscriber {
    type Event = Ev;
    type DataId = Domain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(77)
    }

    fn name(&self) -> &'static str {
        "rollback-failing"
    }

    fn requires(&self) -> &'static [Self::DataId] {
        &[]
    }

    fn provides(&self) -> &'static [Self::DataId] {
        &[]
    }

    fn on_event(&mut self, _event: &Self::Event) {}

    fn on_checkpoint(
        &mut self,
        _barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        Err(SignalError::internal("rollback failure injection"))
    }
}

#[derive(Clone)]
struct ContextWriter {
    seen: Arc<Mutex<Vec<u32>>>,
    staged: Option<u32>,
}

impl EventSubscriber for ContextWriter {
    type Event = Ev;
    type DataId = Domain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(78)
    }

    fn name(&self) -> &'static str {
        "rollback-context-writer"
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

    fn on_event(&mut self, _event: &Self::Event) {
        self.seen.lock().unwrap().push(match _event {
            Ev::Tick => 1,
        });
        self.staged = Some(self.seen.lock().unwrap().len() as u32);
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
struct FailOnBarrier {
    fail_on: Arc<Mutex<Option<CheckpointBarrier>>>,
}

impl EventSubscriber for FailOnBarrier {
    type Event = Ev;
    type DataId = Domain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(79)
    }

    fn name(&self) -> &'static str {
        "rollback-fail-gate"
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
            Err(SignalError::internal("rollback epoch failure injection"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn failed_commit_cannot_leak_key_registry_growth() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .build();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailingSubscriber))
        .unwrap();
    let tripwire_family = define_keyed_computation(&mut runtime, "tripwire-family", ());
    let before = runtime.config().test_registry_counts();
    let mut ctx = ();

    let err = runtime
        .transaction(&mut ctx, |tx| {
            let keyed_def = tripwire_family.keyed("tripwire-key");
            let keyed = keyed_def.node_in_transaction(tx);
            let computation = keyed_def.memoized("tripwire");
            tx.evaluate_keyed(keyed, &computation, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            tx.emit_event(Ev::Tick);
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            Ok(())
        })
        .unwrap_err();

    assert!(format!("{err}").contains("event bus flush failed"));
    assert_eq!(runtime.config().test_registry_counts(), before);
}

#[test]
fn failed_commit_preserves_preexisting_memoized_state() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .build();
    let family = define_keyed_computation(&mut runtime, "stable-family", ());
    let keyed_def = family.keyed("stable-key");
    let keyed = keyed_def.node(&mut runtime);
    let computation = keyed_def.memoized("stable");
    let mut ctx = ();
    let compute_calls = AtomicU32::new(0);

    runtime
        .transaction(&mut ctx, |tx| {
            tx.evaluate_keyed(keyed, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailingSubscriber))
        .unwrap();
    let fresh_def = define_keyed_computation(&mut runtime, "fresh-family", ());

    let err = runtime
        .transaction(&mut ctx, |tx| {
            let keyed_def = fresh_def.keyed("fresh-key");
            let other = keyed_def.node_in_transaction(tx);
            let fresh = keyed_def.memoized("fresh");
            tx.evaluate_keyed(other, &fresh, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(7, 0))))
            })?;
            tx.emit_event(Ev::Tick);
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            Ok(())
        })
        .unwrap_err();
    assert!(format!("{err}").contains("event bus flush failed"));

    mark_dirty(runtime.graph_mut(), keyed, ASPECT_A).unwrap();
    runtime
        .transaction(&mut ctx, |tx| {
            tx.evaluate_keyed(keyed, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(9, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 1);
    assert_eq!(
        runtime
            .graph()
            .replay_events()
            .back()
            .map(|frame| frame.kind),
        Some(ReplayEventKind::TransactionCommitted)
    );
}

#[test]
fn failed_later_epoch_keeps_committed_context_and_records_coherent_diagnostics() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .build();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let fail_on = Arc::new(Mutex::new(None));
    runtime
        .event_bus_mut()
        .subscribe(Box::new(ContextWriter {
            seen: seen.clone(),
            staged: None,
        }))
        .unwrap();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailOnBarrier {
            fail_on: fail_on.clone(),
        }))
        .unwrap();
    let mut ctx = ();

    runtime
        .transaction(&mut ctx, |tx| {
            tx.emit_event(Ev::Tick);
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

    let replay_len_before = runtime.graph().replay_events().len();
    *fail_on.lock().unwrap() = Some(CheckpointBarrier::PerCommit);
    let err = runtime
        .transaction(&mut ctx, |tx| {
            tx.emit_event(Ev::Tick);
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            tx.emit_event(Ev::Tick);
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
        Some(&2),
        "successful earlier epoch in the failed transaction must remain committed",
    );
    assert!(
        runtime
            .event_bus()
            .context()
            .staged::<u32>(Domain::Audit)
            .is_none(),
        "failed later epoch must not leak staged subscriber context",
    );
    let failure = runtime.observe().latest_failure_diagnostics().unwrap();
    assert_eq!(failure.phase, ExecutionFailurePhase::CommitPromotion);
    assert_eq!(failure.event_epochs.len(), 2);
    assert_eq!(
        failure.event_epochs[0].outcome,
        EventEpochOutcome::Committed
    );
    assert_eq!(failure.event_epochs[1].outcome, EventEpochOutcome::Failed);
    let rollback = runtime.observe().latest_rollback_diagnostics().unwrap();
    assert!(rollback
        .reason
        .as_deref()
        .unwrap_or_default()
        .contains("event bus flush failed"));
    assert_eq!(rollback.event_epochs.len(), 2);
    let replay = runtime.graph().replay_events();
    assert_eq!(replay.len(), replay_len_before + 2);
    assert_eq!(
        replay.iter().rev().nth(1).unwrap().kind,
        ReplayEventKind::TransactionRolledBack
    );
    assert_eq!(
        replay.back().unwrap().kind,
        ReplayEventKind::FailureRecorded
    );
}
