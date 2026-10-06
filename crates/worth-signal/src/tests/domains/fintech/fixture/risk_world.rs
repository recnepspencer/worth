use crate::facade::{AspectVersion, NodeId, SignalError};
use crate::tests::support::DependencyBatchBuilder;

use super::super::node_families::{bounded, FintechRuntime};
use super::super::scales::FintechScale;
use super::portfolio_world::PortfolioWorld;

pub(super) struct RiskWorld {
    pub(super) scenario_aggregates: Vec<NodeId>,
    pub(super) bucket_aggregates: Vec<NodeId>,
}

pub(in crate::tests::domains::fintech) fn build_risk_world(
    runtime: &mut FintechRuntime,
    scale: FintechScale,
    portfolio: &PortfolioWorld,
) -> RiskWorld {
    let instruments = &portfolio.instruments;
    let mut scenario_aggregates = Vec::with_capacity(scale.scenarios);
    for scenario_index in 0..scale.scenarios {
        let maximum = instruments.iter().map(|instrument| {
            (
                instrument.scenarios[scenario_index],
                super::super::aspects::RISK,
            )
        });
        let aggregate = runtime
            .graph_mut()
            .node()
            .with_contract(bounded(maximum))
            .reads_aspects(super::super::aspects::full_mask())
            .tolerance(5)
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        for instrument in instruments {
            dependencies
                .append_dependency(
                    aggregate,
                    instrument.scenarios[scenario_index],
                    super::super::aspects::RISK,
                )
                .unwrap();
        }
        dependencies.commit().unwrap();
        scenario_aggregates.push(aggregate);
    }

    let mut bucket_aggregates = Vec::with_capacity(scale.buckets);
    for bucket_index in 0..scale.buckets {
        let maximum = instruments.iter().map(|instrument| {
            (
                instrument.buckets[bucket_index],
                super::super::aspects::RISK,
            )
        });
        let aggregate = runtime
            .graph_mut()
            .node()
            .with_contract(bounded(maximum))
            .reads_aspects(super::super::aspects::full_mask())
            .tolerance(5)
            .build();
        let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
        for instrument in instruments {
            dependencies
                .append_dependency(
                    aggregate,
                    instrument.buckets[bucket_index],
                    super::super::aspects::RISK,
                )
                .unwrap();
        }
        dependencies.commit().unwrap();
        bucket_aggregates.push(aggregate);
    }
    RiskWorld {
        scenario_aggregates,
        bucket_aggregates,
    }
}

impl super::FintechWorld {
    pub(in crate::tests::domains::fintech) fn main_risk_node(&self) -> NodeId {
        self.handles.primary.risk
    }

    pub(in crate::tests::domains::fintech) fn primary_threshold_node(&self) -> NodeId {
        self.handles.primary.threshold
    }

    pub(in crate::tests::domains::fintech) fn read_primary_threshold(
        &mut self,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.primary_threshold_node())
    }

    pub(in crate::tests::domains::fintech) fn read_bucket_aggregate(
        &mut self,
        index: usize,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.bucket_aggregates[index])
    }

    pub(in crate::tests::domains::fintech) fn read_scenario_aggregate(
        &mut self,
        index: usize,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.scenario_aggregates[index])
    }
}
