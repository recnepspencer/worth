//! Native readiness/effect/session boundaries with selected-owner resource custody.
use super::*;
use crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::progression::{
    InvalidationReadinessEpoch, InvalidationStageOrder, ReadyInvalidationBatch,
};
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::facade::{mark_dirty, EvaluationRequestMode, SignalRuntimePolicy};
use crate::logic::invalidation::scheduling::{
    admit_current_readiness, execute_ready, execute_ready_with_work, lower_current_work,
};
use crate::logic::transaction::SignalObservationRequest;
use crate::runtime_policy::SignalConditionalEvaluationBudget;
use crate::tests::support::{version_ab, ASPECT_A};
const BYTES: u64 = 64 * 1024 * 1024;

fn fixture() -> (
    SignalGraph,
    SignalEvaluationPartition,
    Arc<Ledger>,
    [NodeId; 2],
) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let nodes = [graph.create_node(), graph.create_node()];
    let plan = graph
        .build_evaluation_plan(&nodes, EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&plan, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
        .unwrap();
    for node in nodes {
        mark_dirty(&mut graph, node, ASPECT_A).unwrap();
    }
    let ledger = Ledger::new(
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 2,
            maximum_retained_bytes: BYTES,
            maximum_attempt_visits: 8_000_000,
        },
        crate::runtime_policy::SignalConditionalTemporalBudget {
            maximum_live_partitions: 1,
            maximum_reserved_active_wakes: 1,
        },
    );
    let basis = crate::branch::owner_services::conditional_execution::SignalRetainedExecutionBasis::capture(&mut graph, &ledger, &mut Work::new(100_000)).unwrap();
    let slot = basis
        .new_evaluation_partition(&mut Work::new(100_000))
        .unwrap();
    (graph, slot, ledger, nodes)
}
fn ready(
    graph: &SignalGraph,
    node: NodeId,
    epoch: InvalidationReadinessEpoch,
) -> ReadyInvalidationBatch {
    let crate::data::proof::invalidation::revalidation::NodeInvalidationInput::Resolved(input) =
        graph.node_invalidation_input(node).unwrap()
    else {
        panic!("native source seed must resolve");
    };
    let order = InvalidationStageOrder {
        stage: 0,
        order: node.index(),
    };
    admit_current_readiness(
        graph,
        lower_current_work(graph, node, input, epoch, order).unwrap(),
        epoch,
        order,
    )
    .unwrap()
}
fn fill(ledger: &Arc<Ledger>) -> Reservation {
    let available = BYTES - ledger.usage().1 - std::mem::size_of::<Reservation>() as u64;
    ledger
        .reserve(0, Charge::capacity::<u8>(available as usize).unwrap())
        .unwrap()
}

#[test]
fn performed_capture_denials_precede_provider_contact() {
    let (mut graph, mut slot, ledger, nodes) = fixture();
    slot.execute(&mut graph, |selected| {
        let session = selected
            .begin_observation_session(SignalObservationRequest::operation())
            .unwrap();
        let epoch = selected.begin_invalidation_readiness_epoch();
        let calls = std::cell::Cell::new(0);
        let usage = ledger.usage();
        let denied = execute_ready_with_work(
            selected,
            ready(selected, nodes[0], epoch),
            &mut EvaluationWork::Conditional(&mut Work::new(0)),
            || {
                calls.set(calls.get() + 1);
                Ok(())
            },
        );
        assert!(matches!(
            denied,
            Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
        ));
        assert_eq!(ledger.usage(), usage);
        let pressure = fill(&ledger);
        let denied = execute_ready(selected, ready(selected, nodes[0], epoch), || {
            calls.set(calls.get() + 1);
            Ok(())
        });
        assert!(matches!(
            denied,
            Err(SignalError::EvaluationStorageCapacityExhausted)
        ));
        drop(pressure);
        ledger.close();
        let denied = execute_ready(selected, ready(selected, nodes[0], epoch), || {
            calls.set(calls.get() + 1);
            Ok(())
        });
        assert!(matches!(
            denied,
            Err(SignalError::EvaluationStorageUnavailable)
        ));
        assert_eq!(calls.get(), 0);
        assert_eq!(selected.completed_observation_execution_boundaries(), 0);
        assert!(selected
            .finish_optional_observation_session_with_work(
                &session,
                &mut EvaluationWork::Conditional(&mut Work::new(0))
            )
            .unwrap()
            .is_none());
    })
    .unwrap();
}

#[test]
fn nested_performed_capture_releases_locks_and_receipt_retains_its_storage() {
    let (mut graph, mut slot, ledger, nodes) = fixture();
    let receipt = slot
        .execute(&mut graph, |selected| {
            let session = selected
                .begin_observation_session(SignalObservationRequest::operation())
                .unwrap();
            let epoch = selected.begin_invalidation_readiness_epoch();
            execute_ready(selected, ready(selected, nodes[0], epoch), || {
                execute_ready(selected, ready(selected, nodes[1], epoch), || Ok(()))
            })
            .unwrap();
            assert_eq!(selected.invalidation_performed_work().len(), 2);
            let receipt = selected
                .finish_optional_observation_session_with_work(
                    &session,
                    &mut EvaluationWork::Conditional(&mut Work::new(100_000)),
                )
                .unwrap()
                .unwrap();
            for node in nodes {
                assert!(receipt.retains_executed_target(selected.runtime_instance_id(), node));
            }
            receipt
        })
        .unwrap();
    drop(slot);
    drop(graph);
    assert_eq!(ledger.usage().0, 0);
    assert!(ledger.usage().1 > 0);
    drop(receipt);
    assert_eq!(ledger.usage(), (0, 0));
}

#[test]
fn performed_capture_cancellation_and_unwind_release_pending_records() {
    let (mut graph, mut slot, _ledger, nodes) = fixture();
    slot.execute(&mut graph, |selected| {
        let session = selected
            .begin_observation_session(SignalObservationRequest::operation())
            .unwrap();
        let epoch = selected.begin_invalidation_readiness_epoch();
        execute_ready(selected, ready(selected, nodes[0], epoch), || {
            selected.cancel_observation_session(&session).unwrap();
            Ok(())
        })
        .unwrap();
        assert!(selected.invalidation_performed_work().is_empty());
        drop(session);
        let session = selected
            .begin_observation_session(SignalObservationRequest::operation())
            .unwrap();
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: Result<(), SignalError> =
                execute_ready(selected, ready(selected, nodes[0], epoch), || {
                    panic!("provider unwind")
                });
        }));
        assert!(unwind.is_err());
        assert_eq!(
            selected
                .invalidation_performed_work
                .shared_bindings()
                .lock()
                .unwrap()
                .pending,
            0
        );
        execute_ready(selected, ready(selected, nodes[1], epoch), || Ok(())).unwrap();
        assert_eq!(selected.invalidation_performed_work().len(), 1);
        assert!(selected
            .finish_optional_observation_session_with_work(
                &session,
                &mut EvaluationWork::Conditional(&mut Work::new(100_000))
            )
            .unwrap()
            .is_some());
    })
    .unwrap();
}

#[test]
fn performed_receipt_capacity_denial_preserves_execution_for_finalization_retry() {
    let (mut graph, mut slot, ledger, nodes) = fixture();
    slot.execute(&mut graph, |selected| {
        let session = selected
            .begin_observation_session(SignalObservationRequest::operation())
            .unwrap();
        let epoch = selected.begin_invalidation_readiness_epoch();
        let calls = std::cell::Cell::new(0);
        execute_ready(selected, ready(selected, nodes[0], epoch), || {
            calls.set(calls.get() + 1);
            Ok(())
        })
        .unwrap();
        let pressure = fill(&ledger);
        let result = selected.finish_optional_observation_session_with_work(
            &session,
            &mut EvaluationWork::Conditional(&mut Work::new(100_000)),
        );
        assert!(matches!(
            result,
            Err(SignalError::EvaluationStorageCapacityExhausted)
        ));
        assert_ne!(selected.observation_session_active_generation(), 0);
        assert_eq!(selected.invalidation_performed_work().len(), 1);
        drop(pressure);
        assert!(selected
            .finish_optional_observation_session_with_work(
                &session,
                &mut EvaluationWork::Conditional(&mut Work::new(100_000))
            )
            .unwrap()
            .is_some());
        assert_eq!(calls.get(), 1);
    })
    .unwrap();
}

#[test]
fn cancelled_counter_only_session_cannot_receive_a_later_effect_completion() {
    let (mut graph, mut slot, _ledger, nodes) = fixture();
    slot.execute(&mut graph, |selected| {
        let session = selected
            .begin_observation_session(SignalObservationRequest::counters())
            .unwrap();
        let epoch = selected.begin_invalidation_readiness_epoch();
        execute_ready(selected, ready(selected, nodes[0], epoch), || {
            selected.cancel_observation_session(&session).unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!(selected.completed_observation_execution_boundaries(), 0);
        assert_eq!(
            selected
                .invalidation_performed_counters()
                .value(crate::data::telemetry::InvalidationPerformedCounter::NodesEvaluated),
            0
        );
        drop(session);
        let next = selected
            .begin_observation_session(SignalObservationRequest::counters())
            .unwrap();
        assert!(selected
            .finish_optional_observation_session_with_work(
                &next,
                &mut EvaluationWork::Conditional(&mut Work::new(0))
            )
            .unwrap()
            .is_none());
    })
    .unwrap();
}

#[test]
fn superseded_capture_generation_cannot_receive_old_completion_or_counters() {
    for request in [
        SignalObservationRequest::operation(),
        SignalObservationRequest::counters(),
    ] {
        let (mut graph, mut slot, _ledger, nodes) = fixture();
        slot.execute(&mut graph, |selected| {
            let session = selected.begin_observation_session(request).unwrap();
            let epoch = selected.begin_invalidation_readiness_epoch();
            execute_ready(selected, ready(selected, nodes[0], epoch), || {
                selected.cancel_observation_session(&session).unwrap();
                // Explicit runtime session-state transition fixture. Public
                // graph session admission needs &mut and is unavailable inside
                // this shared callback; no replacement public token is forged.
                let successor = selected.observation_sessions.begin(request);
                assert_ne!(successor, session.generation());
                Ok(())
            })
            .unwrap();
            assert_eq!(selected.completed_observation_execution_boundaries(), 0);
            assert_eq!(
                selected
                    .invalidation_performed_counters()
                    .value(crate::data::telemetry::InvalidationPerformedCounter::NodesEvaluated),
                0
            );
            assert!(selected.invalidation_performed_work().is_empty());
            selected
                .observation_sessions
                .finish(selected.observation_session_active_generation());
        })
        .unwrap();
    }
}
