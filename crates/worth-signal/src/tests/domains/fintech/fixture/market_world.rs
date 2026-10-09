use crate::facade::{AspectVersion, ChangedRegion, NodeEvaluationResult, NodeId, SignalError};

use super::super::market_seed::MarketSeed;
use super::super::node_families::{
    build_bucket_sources, build_fx_nodes, build_partition_locality_nodes, build_scenario_sources,
    FintechRuntime, FxNodes,
};
use super::super::partition_surface::{MarketPartition, PartitionDetail, PartitionSurfaceNodes};
use super::super::regimes::MarketRegime;
use super::super::scales::FintechScale;
use super::super::world::FinancialWorldDefinition;

pub(super) struct MarketWorld {
    pub(super) fx: FxNodes,
    pub(super) partition: PartitionSurfaceNodes,
    pub(super) curve_buckets: Vec<NodeId>,
    pub(super) vol_surface_buckets: Vec<NodeId>,
    pub(super) scenario_sources: Vec<NodeId>,
}

pub(in crate::tests::domains::fintech) fn build_market_world(
    runtime: &mut FintechRuntime,
    scale: FintechScale,
) -> MarketWorld {
    let fx = build_fx_nodes(runtime);
    let partition = build_partition_locality_nodes(runtime);
    let curve_buckets = build_bucket_sources(runtime, scale.buckets);
    let vol_surface_buckets = build_bucket_sources(runtime, scale.buckets);
    let scenario_sources = build_scenario_sources(runtime, scale.scenarios);
    let partition = PartitionSurfaceNodes {
        market_regions: partition.market_regions,
        rates_partition: partition.rates_partition,
        credit_partition: partition.credit_partition,
        rates_bucket_zero: partition.rates_bucket_zero,
        coarse_book: partition.coarse_book,
    };
    MarketWorld {
        fx,
        partition,
        curve_buckets,
        vol_surface_buckets,
        scenario_sources,
    }
}

fn apply_signed_delta(base: u64, delta: i64) -> u64 {
    if delta >= 0 {
        base.saturating_add(delta as u64)
    } else {
        base.saturating_sub(delta.unsigned_abs())
    }
}

pub(in crate::tests::domains::fintech) fn seed_partition_baseline(
    runtime: &mut FintechRuntime,
    market_regions: NodeId,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(market_regions, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([
                        (super::super::aspects::PRICE, 0),
                        (super::super::aspects::RISK, 0),
                    ]))
                    .with_output_identity("partition-market-baseline"),
                ))
            })?;
            Ok(())
        })
        .expect("partition locality source should seed cleanly");
}

impl super::FintechWorld {
    pub(in crate::tests::domains::fintech) fn seed_regime(
        &mut self,
        regime: MarketRegime,
        seed: u64,
    ) -> Result<(), SignalError> {
        self.seed_financial_definition(FinancialWorldDefinition::runtime_fixture(
            self.financial_definition.fixture_scale(),
            regime,
            seed,
        ))
    }

    pub(in crate::tests::domains::fintech) fn seed_market(
        &mut self,
        market_seed: MarketSeed,
    ) -> Result<(), SignalError> {
        self.seed_regime(market_seed.regime, market_seed.seed)
    }

    pub(in crate::tests::domains::fintech) fn seed_financial_definition(
        &mut self,
        definition: FinancialWorldDefinition,
    ) -> Result<(), SignalError> {
        assert_eq!(
            definition.fixture_scale(),
            self.financial_definition.fixture_scale(),
            "financial runtime fixture scale is immutable after topology compilation"
        );
        let revision = if self.market_revision == 0 {
            1
        } else if self.financial_definition == definition {
            self.market_revision
        } else {
            self.market_revision
                .checked_add(1)
                .expect("financial fixture semantic revision overflow")
        };
        super::super::market_state::seed_financial_definition(self, &definition, revision)?;
        self.financial_definition = definition;
        self.market_revision = revision;
        Ok(())
    }

    pub(in crate::tests::domains::fintech) fn primary_market_source(&self) -> NodeId {
        self.handles.primary.market_source
    }

    pub(in crate::tests::domains::fintech) fn partitioned_market_source(&self) -> NodeId {
        self.handles.partition.market_regions
    }

    pub(in crate::tests::domains::fintech) fn rates_partition_node(&self) -> NodeId {
        self.handles.partition.rates_partition
    }

    pub(in crate::tests::domains::fintech) fn credit_partition_node(&self) -> NodeId {
        self.handles.partition.credit_partition
    }

    pub(in crate::tests::domains::fintech) fn rates_bucket_zero_node(&self) -> NodeId {
        self.handles.partition.rates_bucket_zero
    }

    pub(in crate::tests::domains::fintech) fn coarse_partition_book_node(&self) -> NodeId {
        self.handles.partition.coarse_book
    }

    pub(in crate::tests::domains::fintech) fn read_primary_market_source(
        &mut self,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.primary_market_source())
    }

    pub(in crate::tests::domains::fintech) fn inject_primary_market_rollback(
        &mut self,
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

        let top_desk = self.top_desk();
        let evaluation = self.evaluation_shape();
        let evaluator = evaluation.evaluator();
        let source = self.primary_market_source();
        let err = self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(source, super::super::aspects::PRICE)?;
            tx.mark_dirty(source, super::super::aspects::VOL)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([
                        (super::super::aspects::PRICE, 99_999),
                        (super::super::aspects::VOL, 99_999),
                        (super::super::aspects::CURVE, 99_999),
                        (super::super::aspects::LIQUIDITY, 99_999),
                        (super::super::aspects::RISK, 99_999),
                        (super::super::aspects::ALERT, 1),
                    ]))
                    .with_output_identity("bad-branch-correction"),
                ))
            })?;
            tx.read(top_desk, &evaluator)?;
            Err(SignalError::invalid_input("synthetic analysis rollback"))
        });
        match err {
            Ok(_) => Err(SignalError::invalid_input(
                "synthetic rollback unexpectedly committed",
            )),
            Err(_) => Ok(()),
        }
    }

    pub(crate) fn bump_primary_market(
        &mut self,
        price_delta: i64,
        vol_delta: i64,
        curve_delta: i64,
        liquidity_delta: i64,
    ) -> Result<AspectVersion, SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        let source = self.primary_market_source();
        let current = self.runtime.graph().node_aspect_version(source)?;
        let bumped = AspectVersion::from_updates([
            (
                super::super::aspects::PRICE,
                apply_signed_delta(current.get(super::super::aspects::PRICE), price_delta),
            ),
            (
                super::super::aspects::VOL,
                apply_signed_delta(current.get(super::super::aspects::VOL), vol_delta),
            ),
            (
                super::super::aspects::CURVE,
                apply_signed_delta(current.get(super::super::aspects::CURVE), curve_delta),
            ),
            (
                super::super::aspects::LIQUIDITY,
                apply_signed_delta(
                    current.get(super::super::aspects::LIQUIDITY),
                    liquidity_delta,
                ),
            ),
            (
                super::super::aspects::RISK,
                apply_signed_delta(
                    current.get(super::super::aspects::RISK),
                    price_delta + vol_delta,
                ),
            ),
            (
                super::super::aspects::ALERT,
                current.get(super::super::aspects::ALERT),
            ),
        ]);

        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(source, super::super::aspects::PRICE)?;
            tx.mark_dirty(source, super::super::aspects::VOL)?;
            tx.mark_dirty(source, super::super::aspects::CURVE)?;
            tx.mark_dirty(source, super::super::aspects::LIQUIDITY)?;
            tx.mark_dirty(source, super::super::aspects::RISK)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(bumped)
                        .with_output_identity("primary-market-bump"),
                ))
            })?;
            Ok(())
        })?;

        self.read_node(source)
    }

    pub(in crate::tests::domains::fintech) fn apply_partition_shock(
        &mut self,
        partition: MarketPartition,
        detail: Option<PartitionDetail>,
        price_delta: i64,
    ) -> Result<AspectVersion, SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        let source = self.partitioned_market_source();
        let current = self.runtime.graph().node_aspect_version(source)?;
        let bumped = AspectVersion::from_updates([
            (
                super::super::aspects::PRICE,
                apply_signed_delta(current.get(super::super::aspects::PRICE), price_delta),
            ),
            (
                super::super::aspects::RISK,
                apply_signed_delta(current.get(super::super::aspects::RISK), price_delta),
            ),
        ]);
        let changed_region = match detail {
            Some(detail) => ChangedRegion::new(partition.token()).with_detail(detail.token()),
            None => ChangedRegion::new(partition.token()),
        };

        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty_with_regions(
                source,
                super::super::aspects::PRICE,
                std::slice::from_ref(&changed_region),
            )?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(bumped)
                        .with_output_identity(format!(
                            "partition-shock-{}-{}",
                            partition.token(),
                            detail.map(PartitionDetail::token).unwrap_or("whole")
                        ))
                        .with_changed_region(changed_region.clone()),
                ))
            })?;
            Ok(())
        })?;

        self.read_node(source)
    }

    pub(in crate::tests::domains::fintech) fn shock_rates_bucket_zero(
        &mut self,
        price_delta: i64,
    ) -> Result<AspectVersion, SignalError> {
        self.apply_partition_shock(
            MarketPartition::Rates,
            Some(PartitionDetail::Bucket0),
            price_delta,
        )
    }

    pub(in crate::tests::domains::fintech) fn shock_rates_bucket_one(
        &mut self,
        price_delta: i64,
    ) -> Result<AspectVersion, SignalError> {
        self.apply_partition_shock(
            MarketPartition::Rates,
            Some(PartitionDetail::Bucket1),
            price_delta,
        )
    }

    pub(in crate::tests::domains::fintech) fn shock_credit_partition(
        &mut self,
        price_delta: i64,
    ) -> Result<AspectVersion, SignalError> {
        self.apply_partition_shock(MarketPartition::Credit, None, price_delta)
    }

    pub(in crate::tests::domains::fintech) fn read_rates_partition(
        &mut self,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.rates_partition_node())
    }

    pub(in crate::tests::domains::fintech) fn read_credit_partition(
        &mut self,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.credit_partition_node())
    }

    pub(in crate::tests::domains::fintech) fn read_rates_bucket_zero(
        &mut self,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.rates_bucket_zero_node())
    }

    pub(in crate::tests::domains::fintech) fn read_coarse_partition_book(
        &mut self,
    ) -> Result<AspectVersion, SignalError> {
        self.read_node(self.coarse_partition_book_node())
    }
}
