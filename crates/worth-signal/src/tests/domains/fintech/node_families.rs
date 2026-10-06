use crate::facade::*;
use crate::tests::support::DependencyBatchBuilder;

use super::aspects::{full_mask, market_mask, pricing_mask, ALERT};
use super::execution_tier::FintechTier;

mod partition_locality;
mod sources;
pub(super) use partition_locality::build_partition_locality_nodes;
pub(super) use sources::{build_bucket_sources, build_scenario_sources};

pub(super) type FintechRuntime = SignalRuntime<(), (), (), (), FintechTier>;

pub(super) fn bounded(inputs: impl IntoIterator<Item = (NodeId, Aspect)>) -> NodeContract {
    NodeContract::wildcard()
        .with_bounded_inputs(BoundedSignalInputs::new(
            inputs
                .into_iter()
                .map(|(source, aspect)| DeclaredSignalInput::new(source, aspect)),
        ))
        .with_max_checked_result_heap_bytes(super::evaluation::maximum_checked_result_heap_bytes())
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FxNodes {
    pub eur_usd: NodeId,
    pub usd_jpy: NodeId,
    pub eur_jpy: NodeId,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AggregateSourceNodes {
    pub book_state: NodeId,
    pub desk_limit: NodeId,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct InstrumentNodes {
    pub market: NodeId,
    pub normalized: NodeId,
    pub price: NodeId,
    pub risk: NodeId,
    pub alert: NodeId,
    pub threshold: NodeId,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PartitionLocalityNodes {
    pub market_regions: NodeId,
    pub rates_partition: NodeId,
    pub credit_partition: NodeId,
    pub rates_bucket_zero: NodeId,
    pub coarse_book: NodeId,
}

pub(super) fn build_instrument_nodes(runtime: &mut FintechRuntime) -> InstrumentNodes {
    let market = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .reads_aspects(full_mask())
        .build();
    let normalized = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([
            (market, super::aspects::PRICE),
            (market, super::aspects::VOL),
            (market, super::aspects::CURVE),
            (market, super::aspects::LIQUIDITY),
        ]))
        .reads_aspects(market_mask())
        .tolerance(1)
        .build();
    let price = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([
            (normalized, super::aspects::PRICE),
            (normalized, super::aspects::VOL),
            (normalized, super::aspects::CURVE),
        ]))
        .reads_aspects(pricing_mask())
        .tolerance(2)
        .build();
    let risk = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([
            (price, super::aspects::RISK),
            (normalized, super::aspects::LIQUIDITY),
        ]))
        .reads_aspects(pricing_mask())
        .tolerance(3)
        .build();
    let alert = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([(risk, super::aspects::ALERT)]))
        .reads_aspects(full_mask())
        .aspect_filter(ALERT)
        .tolerance(1)
        .build();
    let threshold = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([(price, super::aspects::PRICE)]))
        .reads_aspects(pricing_mask())
        .condition(EvaluationCondition::DeltaThreshold(2.0))
        .tolerance(2)
        .build();

    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_dependency(normalized, market, super::aspects::PRICE)
        .unwrap()
        .append_dependency(normalized, market, super::aspects::VOL)
        .unwrap()
        .append_dependency(normalized, market, super::aspects::CURVE)
        .unwrap()
        .append_dependency(normalized, market, super::aspects::LIQUIDITY)
        .unwrap()
        .append_dependency(price, normalized, super::aspects::PRICE)
        .unwrap()
        .append_dependency(price, normalized, super::aspects::VOL)
        .unwrap()
        .append_dependency(price, normalized, super::aspects::CURVE)
        .unwrap()
        .append_dependency(risk, price, super::aspects::RISK)
        .unwrap()
        .append_dependency(risk, normalized, super::aspects::LIQUIDITY)
        .unwrap()
        .append_dependency(alert, risk, super::aspects::ALERT)
        .unwrap()
        .append_dependency(threshold, price, super::aspects::PRICE)
        .unwrap();
    dependencies.commit().unwrap();

    InstrumentNodes {
        market,
        normalized,
        price,
        risk,
        alert,
        threshold,
    }
}

pub(super) fn build_bucket_exposure_nodes(
    runtime: &mut FintechRuntime,
    instrument: &InstrumentNodes,
    curve_buckets: &[NodeId],
    vol_surface_buckets: &[NodeId],
) -> Vec<NodeId> {
    let mut nodes = Vec::with_capacity(curve_buckets.len());
    for (&curve, &vol) in curve_buckets.iter().zip(vol_surface_buckets) {
        let node = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([
                (instrument.risk, super::aspects::RISK),
                (instrument.threshold, super::aspects::PRICE),
                (curve, super::aspects::CURVE),
                (vol, super::aspects::VOL),
            ]))
            .reads_aspects(pricing_mask())
            .tolerance(3)
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        dependencies
            .append_dependency(node, instrument.risk, super::aspects::RISK)
            .unwrap()
            .append_dependency(node, instrument.threshold, super::aspects::PRICE)
            .unwrap();
        dependencies.commit().unwrap();
        nodes.push(node);
    }
    nodes
}

pub(super) fn build_aggregate_sources(runtime: &mut FintechRuntime) -> AggregateSourceNodes {
    let book_state = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .reads_aspects(super::aspects::full_mask())
        .tolerance(2)
        .build();
    let desk_limit = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .reads_aspects(super::aspects::full_mask())
        .tolerance(2)
        .build();

    AggregateSourceNodes {
        book_state,
        desk_limit,
    }
}

pub(super) fn build_scenario_nodes(
    runtime: &mut FintechRuntime,
    instrument: &InstrumentNodes,
    scenario_sources: &[NodeId],
    scenarios: usize,
) -> Vec<NodeId> {
    let mut nodes = Vec::with_capacity(scenarios);
    for scenario_source in scenario_sources.iter().copied().take(scenarios) {
        let node = runtime
            .graph_mut()
            .node()
            .with_contract(bounded([
                (instrument.price, super::aspects::PRICE),
                (instrument.risk, super::aspects::RISK),
                (instrument.alert, super::aspects::ALERT),
                (scenario_source, super::aspects::RISK),
                (scenario_source, super::aspects::VOL),
            ]))
            .reads_aspects(full_mask())
            .tolerance(4)
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        dependencies
            .append_dependency(node, instrument.price, super::aspects::PRICE)
            .unwrap()
            .append_dependency(node, instrument.risk, super::aspects::RISK)
            .unwrap()
            .append_dependency(node, instrument.alert, super::aspects::ALERT)
            .unwrap()
            .append_dependency(node, scenario_source, super::aspects::RISK)
            .unwrap()
            .append_dependency(node, scenario_source, super::aspects::VOL)
            .unwrap();
        dependencies.commit().unwrap();
        nodes.push(node);
    }
    nodes
}

pub(super) fn build_fx_nodes(runtime: &mut FintechRuntime) -> FxNodes {
    let eur_usd = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .reads_aspects(super::aspects::full_mask())
        .tolerance(1)
        .build();
    let usd_jpy = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .reads_aspects(super::aspects::full_mask())
        .tolerance(1)
        .build();
    let eur_jpy = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([
            (eur_usd, super::aspects::PRICE),
            (usd_jpy, super::aspects::PRICE),
        ]))
        .reads_aspects(super::aspects::full_mask())
        .tolerance(2)
        .build();
    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_dependency(eur_jpy, eur_usd, super::aspects::PRICE)
        .unwrap()
        .append_dependency(eur_jpy, usd_jpy, super::aspects::PRICE)
        .unwrap();
    dependencies.commit().unwrap();
    FxNodes {
        eur_usd,
        usd_jpy,
        eur_jpy,
    }
}
