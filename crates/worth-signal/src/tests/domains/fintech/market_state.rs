use crate::facade::*;

use super::aspects::{ALERT, CURVE, LIQUIDITY, PRICE, RISK, VOL};
use super::fixture::FintechDomainFixture;
use super::world::{
    FinancialFixtureProjection, FinancialWorldDefinition, FixtureAggregateState,
    FixtureMarketPoint, FixtureScenarioShock,
};

fn point_version(revision: u64) -> AspectVersion {
    AspectVersion::from_updates([
        (PRICE, revision),
        (VOL, revision),
        (CURVE, revision),
        (LIQUIDITY, revision),
        (RISK, revision),
        (ALERT, revision),
    ])
}

fn aggregate_version(revision: u64) -> AspectVersion {
    AspectVersion::from_updates([(RISK, revision), (ALERT, revision)])
}

fn scenario_version(revision: u64) -> AspectVersion {
    AspectVersion::from_updates([(RISK, revision), (VOL, revision)])
}

fn point_identity(point: FixtureMarketPoint) -> String {
    format!(
        "price={};vol={};curve={};liquidity={};risk={};alert={}",
        point.price, point.vol, point.curve, point.liquidity, point.risk, point.alert
    )
}

fn aggregate_identity(state: FixtureAggregateState) -> String {
    format!("risk={};alert={}", state.risk, state.alert)
}

fn scenario_identity(shock: FixtureScenarioShock) -> String {
    format!("risk={};vol={}", shock.risk, shock.vol)
}

pub(super) fn seed_financial_definition(
    fixture: &mut FintechDomainFixture,
    definition: &FinancialWorldDefinition,
    revision: u64,
) -> Result<(), SignalError> {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let projection = FinancialFixtureProjection::from_definition(definition);
    let mut ctx = ();
    fixture
        .runtime
        .transaction(request_execution, &mut ctx, |tx| {
            for instrument in &fixture.instruments {
                let point =
                    projection.market_point(instrument.instrument_index, instrument.book_index);
                let label = point_identity(point);
                if revision > 1 {
                    tx.mark_changed(instrument.core.market, PRICE)?;
                }
                tx.read(instrument.core.market, &move |view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(point_version(revision))
                            .with_output_identity(label.clone()),
                    ))
                })?;
            }

            let fx = projection.fx_market();
            if revision > 1 {
                tx.mark_changed(fixture.fx.eur_usd, PRICE)?;
                tx.mark_changed(fixture.fx.usd_jpy, PRICE)?;
            }
            tx.read(fixture.fx.eur_usd, &move |view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        PRICE, revision,
                    )]))
                    .with_output_identity(format!("eur-usd={}", fx.eur_usd)),
                ))
            })?;
            tx.read(fixture.fx.usd_jpy, &move |view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        PRICE, revision,
                    )]))
                    .with_output_identity(format!("usd-jpy={}", fx.usd_jpy)),
                ))
            })?;

            let curve_series = projection.curve_bucket_series(fixture.curve_buckets.len());
            for (index, node) in fixture.curve_buckets.iter().enumerate() {
                let value = curve_series[index];
                if revision > 1 {
                    tx.mark_changed(*node, CURVE)?;
                }
                tx.read(*node, &move |view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                            CURVE, revision,
                        )]))
                        .with_output_identity(format!("curve-{index}={value}")),
                    ))
                })?;
            }

            let vol_series = projection.vol_surface_series(fixture.vol_surface_buckets.len());
            for (index, node) in fixture.vol_surface_buckets.iter().enumerate() {
                let value = vol_series[index];
                if revision > 1 {
                    tx.mark_changed(*node, VOL)?;
                }
                tx.read(*node, &move |view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                            VOL, revision,
                        )]))
                        .with_output_identity(format!("surface-{index}={value}")),
                    ))
                })?;
            }

            let scenario_shocks = projection.scenario_shocks(fixture.scenario_sources.len());
            for (index, node) in fixture.scenario_sources.iter().enumerate() {
                let shock = scenario_shocks[index];
                if revision > 1 {
                    tx.mark_changed(*node, RISK)?;
                }
                tx.read(*node, &move |view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(scenario_version(revision))
                            .with_output_identity(format!(
                                "scenario-shock-{index}:{}",
                                scenario_identity(shock)
                            )),
                    ))
                })?;
            }

            let book_states = projection.book_states(fixture.book_aggregates.len());
            for (index, sources) in fixture
                .aggregate_sources
                .iter()
                .take(fixture.book_aggregates.len())
                .enumerate()
            {
                let state = book_states[index];
                if revision > 1 {
                    tx.mark_changed(sources.book_state, RISK)?;
                }
                tx.read(sources.book_state, &move |view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(aggregate_version(revision))
                            .with_output_identity(format!(
                                "book-state-{index}:{}",
                                aggregate_identity(state)
                            )),
                    ))
                })?;
            }

            let desk_limits = projection.desk_limits(fixture.desk_aggregates.len());
            for (index, sources) in fixture
                .aggregate_sources
                .iter()
                .take(fixture.desk_aggregates.len())
                .enumerate()
            {
                let state = desk_limits[index];
                if revision > 1 {
                    tx.mark_changed(sources.desk_limit, RISK)?;
                }
                tx.read(sources.desk_limit, &move |view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(aggregate_version(revision))
                            .with_output_identity(format!(
                                "desk-limit-{index}:{}",
                                aggregate_identity(state)
                            )),
                    ))
                })?;
            }
            Ok(())
        })?;
    Ok(())
}
