use super::*;

#[test]
fn standing_demand_settles_a_reached_node_without_any_observer() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second, chain.unrelated], &evaluator);
    let before = *runtime.telemetry();

    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_demand(&evaluator, &[chain.second, chain.unrelated])?);
            Ok(())
        })
        .unwrap();

    // Reach is source, first, second. `unrelated` is standing demand but not
    // reached by this change, so it is not a target and never recomputed.
    assert_eq!(
        summary.expect("transaction closure ran"),
        ObservedDemandSummary {
            reach_visits: 3,
            targets: 1,
            passes: 2,
            tasks_executed: 2,
        }
    );
    assert_eq!(node_state(&runtime, chain.first), NodeState::Clean);
    assert_eq!(node_state(&runtime, chain.second), NodeState::Clean);
    assert_eq!(calls_for(&calls, chain.first), 2);
    assert_eq!(calls_for(&calls, chain.second), 2);
    assert_eq!(calls_for(&calls, chain.unrelated), 1);
    let after = *runtime.telemetry();
    assert_eq!(
        after.transaction.observed_demand_targets - before.transaction.observed_demand_targets,
        1
    );
}

#[test]
fn standing_demand_that_the_change_does_not_reach_is_not_demanded() {
    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second, chain.unrelated], &evaluator);

    let mut summary = None;
    runtime
        .transaction(&mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_demand(&evaluator, &[chain.unrelated])?);
            Ok(())
        })
        .unwrap();

    assert_eq!(
        summary.expect("transaction closure ran"),
        ObservedDemandSummary {
            reach_visits: 3,
            targets: 0,
            passes: 0,
            tasks_executed: 0,
        }
    );
    // Nothing demanded `first`, so it stays deferred like any on-demand node.
    assert_eq!(node_state(&runtime, chain.first), NodeState::Dirty);
    assert_eq!(calls_for(&calls, chain.first), 1);
    assert_eq!(calls_for(&calls, chain.unrelated), 1);
}
