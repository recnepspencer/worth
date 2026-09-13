use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::data::checkpoint::CheckpointBarrier;
use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
use crate::data::subscriber_context::SubscriberContext;
use crate::diagnostics::replay::ReplayEventKind;
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

#[derive(Clone)]
struct RecordingSubscriber {
    id: u32,
    name: &'static str,
    log: Arc<Mutex<Vec<&'static str>>>,
    fail_on_checkpoint: bool,
    requires_audit: bool,
    provides_audit: bool,
}

impl EventSubscriber for RecordingSubscriber {
    type Event = Ev;
    type DataId = Domain;
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(self.id)
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn requires(&self) -> &'static [Self::DataId] {
        if self.requires_audit {
            &[Domain::Audit]
        } else {
            &[]
        }
    }

    fn provides(&self) -> &'static [Self::DataId] {
        if self.provides_audit {
            &[Domain::Audit]
        } else {
            &[]
        }
    }

    fn on_event(&mut self, _event: &Self::Event) {}

    fn on_checkpoint(
        &mut self,
        _barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<Self::DataId>,
        _runtime: &mut Self::RuntimeContext,
    ) -> Result<(), SignalError> {
        self.log.lock().unwrap().push(self.name);
        if self.fail_on_checkpoint {
            Err(SignalError::internal("checkpoint failure"))
        } else {
            Ok(())
        }
    }

    fn on_rollback(&mut self, _runtime: &mut Self::RuntimeContext) {
        self.log.lock().unwrap().push(match self.name {
            "first" => "first-rollback",
            "second" => "second-rollback",
            other => other,
        });
    }
}

#[test]
fn failed_event_flush_triggers_compensating_rollbacks_for_flushed_subscribers() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .build();
    let node = runtime.graph_mut().node().build();
    let log = Arc::new(Mutex::new(Vec::new()));
    runtime
        .event_bus_mut()
        .subscribe(Box::new(RecordingSubscriber {
            id: 1,
            name: "first",
            log: log.clone(),
            fail_on_checkpoint: false,
            requires_audit: false,
            provides_audit: true,
        }))
        .unwrap();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(RecordingSubscriber {
            id: 2,
            name: "second",
            log: log.clone(),
            fail_on_checkpoint: true,
            requires_audit: true,
            provides_audit: false,
        }))
        .unwrap();

    let before = runtime.graph().get_state(node).unwrap();
    let mut ctx = ();
    let mut tx = runtime.begin(&mut ctx);
    tx.mark_dirty(node, ASPECT_A).unwrap();
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();

    let err = tx.commit().unwrap_err();
    assert!(format!("{err}").contains("event bus flush failed"));
    assert_eq!(runtime.graph().get_state(node).unwrap(), before);

    let log = log.lock().unwrap().clone();
    assert!(log.starts_with(&["first", "second"]));
    assert!(log.contains(&"first-rollback"));
    assert!(log.contains(&"second-rollback"));
}

#[test]
fn failed_commit_discards_staged_key_registry_growth_and_created_keyed_nodes() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .build();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(RecordingSubscriber {
            id: 1,
            name: "first",
            log: Arc::new(Mutex::new(Vec::new())),
            fail_on_checkpoint: true,
            requires_audit: false,
            provides_audit: false,
        }))
        .unwrap();

    let rollback_family = define_keyed_computation(&mut runtime, "rollback-fresh-family", ());
    let before_counts = runtime.config().test_registry_counts();
    let before_active = runtime.graph().active_node_count();
    let before_replay_len = runtime.graph().replay_events().len();
    let mut ctx = ();

    let key_name = "rollback-fresh-key";
    let memo_name = "rollback-fresh-memo";

    let err = {
        let mut tx = runtime.begin(&mut ctx);
        let keyed_def = rollback_family.keyed(key_name);
        let keyed = keyed_def.node_in_transaction(&mut tx);
        let computation = keyed_def.memoized(memo_name);
        tx.evaluate_keyed(keyed, &computation, &|view| {
            Ok(view.finish(
                NodeEvaluationResult::from_version(version_ab(7, 0))
                    .with_output_identity("rollback-artifact"),
            ))
        })
        .unwrap();
        tx.emit_event(Ev::Tick);
        tx.flush_events(CheckpointBarrier::PerOperation).unwrap();
        let err = tx.commit().unwrap_err();
        assert!(!runtime.graph().is_alive(keyed));
        err
    };

    assert!(format!("{err}").contains("event bus flush failed"));
    assert_eq!(
        runtime.config().test_registry_counts(),
        before_counts,
        "failed commit must restore family/key/memo registry and keyed-node maps to baseline",
    );
    assert_eq!(
        runtime.graph().active_node_count(),
        before_active,
        "failed commit must remove transaction-created keyed nodes",
    );

    let replay = runtime.graph().replay_events();
    assert_eq!(replay.len(), before_replay_len + 2);
    assert_eq!(
        replay.iter().rev().nth(1).unwrap().kind,
        ReplayEventKind::TransactionRolledBack
    );
    assert_eq!(
        replay.back().unwrap().kind,
        ReplayEventKind::FailureRecorded
    );
}

#[test]
fn failed_commit_preserves_preexisting_memo_cache_while_discarding_new_staged_growth() {
    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_domains::<Domain>()
        .with_events::<Ev>()
        .checkpoint_barrier(CheckpointBarrier::PerOperation)
        .build();

    let stable_family = define_keyed_computation(&mut runtime, "stable-family", ());
    let stable_keyed_def = stable_family.keyed("stable-key");
    let stable_keyed = stable_keyed_def.node(&mut runtime);
    let stable_computation = stable_keyed_def.memoized("stable-memo");
    let stable_compute_calls = AtomicU32::new(0);
    let mut ctx = ();

    runtime
        .transaction(&mut ctx, |tx| {
            tx.evaluate_keyed(stable_keyed, &stable_computation, &|view| {
                stable_compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("stable-artifact"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let fresh_def = define_keyed_computation(&mut runtime, "fresh-family", ());
    let baseline_counts = runtime.config().test_registry_counts();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(RecordingSubscriber {
            id: 9,
            name: "failing",
            log: Arc::new(Mutex::new(Vec::new())),
            fail_on_checkpoint: true,
            requires_audit: false,
            provides_audit: false,
        }))
        .unwrap();

    let err = runtime
        .transaction(&mut ctx, |tx| {
            let keyed_def = fresh_def.keyed("fresh-key");
            let keyed = keyed_def.node_in_transaction(tx);
            let fresh = keyed_def.memoized("fresh");
            tx.evaluate_keyed(keyed, &fresh, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(9, 0))
                        .with_output_identity("fresh-artifact"),
                ))
            })?;
            tx.emit_event(Ev::Tick);
            tx.flush_events(CheckpointBarrier::PerOperation)?;
            Ok(())
        })
        .unwrap_err();

    assert!(format!("{err}").contains("event bus flush failed"));
    assert_eq!(
        runtime.config().test_registry_counts(),
        baseline_counts,
        "failed commit must discard fresh registry and memo growth without damaging committed memo state",
    );

    mark_dirty(runtime.graph_mut(), stable_keyed, ASPECT_A).unwrap();
    runtime
        .transaction(&mut ctx, |tx| {
            tx.evaluate_keyed(stable_keyed, &stable_computation, &|view| {
                stable_compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(99, 0))
                        .with_output_identity("should-not-run"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(
        stable_compute_calls.load(Ordering::Relaxed),
        1,
        "baseline memoized result must survive failed commits and remain reusable afterward",
    );
    let metrics = runtime.observe().metrics();
    assert!(metrics.evaluation.memoization_hits >= 1);
    assert_eq!(
        runtime
            .graph()
            .replay_events()
            .back()
            .map(|event| event.kind),
        Some(ReplayEventKind::TransactionCommitted)
    );
}
