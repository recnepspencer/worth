use crate::data::telemetry::InvalidationPerformedCounter;
use crate::tests::domains::fintech::world::FinancialWorldDefinition;

#[test]
fn descriptive_surfaces_are_each_selectable_without_performed_counter_capture() {
    let surfaces = [
        crate::facade::SignalObservationRequest::counters(),
        crate::facade::SignalObservationRequest::work(),
        crate::facade::SignalObservationRequest::lineage(),
        crate::facade::SignalObservationRequest::facts(),
        crate::facade::SignalObservationRequest::frontier(),
        crate::facade::SignalObservationRequest::replay(),
        crate::facade::SignalObservationRequest::telemetry(),
    ];
    for request in surfaces {
        let mut compiled = super::compile_financial_locality_world_at_tier(
            FinancialWorldDefinition::convergent_factor_batch(41, 0),
            crate::facade::DiagnosticsTier::Development,
        )
        .unwrap();
        compiled.locality_mut().runtime.set_runtime_policy(
            crate::facade::SignalRuntimePolicy::operational()
                .with_explanation_retention(
                    crate::diagnostics::policy::ArtifactRetentionPolicy::Retain,
                )
                .with_provenance_retention(
                    crate::diagnostics::policy::ArtifactRetentionPolicy::Retain,
                )
                .with_history_details(true)
                .with_observation_activation(
                    worth_foundational::ObservationActivationProfile::OnDemand,
                ),
        );
        let observation = compiled
            .locality_mut()
            .runtime
            .begin_observation_session(request)
            .unwrap();
        let graph = compiled.locality().runtime.graph();
        let before_lineage = graph
            .observe()
            .lineage_records()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let before_replay = graph
            .observe()
            .replay_events()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let before_frontier = graph.observe().latest_frontier_execution_summary().cloned();
        let before_flow = graph
            .observe()
            .latest_flow_diagnostics()
            .map(|flow| flow.to_owned_summary());
        let before_facts = compiled
            .locality()
            .handles
            .values()
            .map(|node| (*node, graph.explanation_fact(*node).cloned()))
            .collect::<Vec<_>>();
        compiled.run_locality_action_trace(0).unwrap();
        let graph = compiled.locality().runtime.graph();
        let work_nonempty = !graph.invalidation_performed_work().is_empty();
        let receipt = compiled
            .locality()
            .runtime
            .finish_observation_session(&observation)
            .unwrap();
        assert_eq!(receipt.request(), request);
        let counters_nonzero = receipt
            .realized_counters()
            .values()
            .into_iter()
            .any(|value| value > 0);
        if !request.includes(crate::facade::SignalObservationSurface::PerformedCounters) {
            assert!(InvalidationPerformedCounter::ALL
                .into_iter()
                .all(|counter| receipt.realized_counters().value(counter) == 0));
        }
        let lineage_nonempty = graph
            .observe()
            .lineage_records()
            .iter()
            .ne(before_lineage.iter());
        let facts_nonempty = compiled
            .locality()
            .handles
            .values()
            .map(|node| (*node, graph.explanation_fact(*node).cloned()))
            .collect::<Vec<_>>()
            != before_facts;
        let frontier_nonempty =
            graph.observe().latest_frontier_execution_summary().cloned() != before_frontier;
        let replay_nonempty = graph
            .observe()
            .replay_events()
            .iter()
            .ne(before_replay.iter());
        let telemetry_nonempty = graph
            .observe()
            .latest_flow_diagnostics()
            .map(|flow| flow.to_owned_summary())
            != before_flow;

        assert_eq!(
            counters_nonzero,
            request.includes(crate::facade::SignalObservationSurface::PerformedCounters)
        );
        assert_eq!(
            work_nonempty,
            request.includes(crate::facade::SignalObservationSurface::PerformedWork)
        );
        assert_eq!(
            lineage_nonempty,
            request.includes(crate::facade::SignalObservationSurface::DescriptiveLineage)
        );
        assert_eq!(
            facts_nonempty,
            request.includes(crate::facade::SignalObservationSurface::DescriptiveFacts)
        );
        assert_eq!(
            frontier_nonempty,
            request.includes(crate::facade::SignalObservationSurface::FrontierTrace)
        );
        assert_eq!(
            replay_nonempty,
            request.includes(crate::facade::SignalObservationSurface::ReplayDetail)
        );
        assert_eq!(
            telemetry_nonempty,
            request.includes(crate::facade::SignalObservationSurface::OptionalTelemetry)
        );

        if request.includes(crate::facade::SignalObservationSurface::DescriptiveFacts) {
            assert!(facts_nonempty);
        }
    }
}
