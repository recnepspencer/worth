mod comparator_execution;
mod compiled_authority;
mod dependency_rewire;
mod evaluation;
mod factor_sequence;
mod inspection;
mod lifecycle_composition;
mod locality_evaluation;
mod locality_execution;
mod locality_topology;
mod quote_translation;
mod runtime_finance;
pub(super) mod topology;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::facade::{NodeState, RuntimeMetrics, SignalGraph, SignalRuntime};

use super::{
    FinancialConsumerRole, FinancialEconomicSnapshot, FinancialSemanticProjection,
    FinancialWorldDefinition, InstrumentId, MarketFactorKey, SemanticOutputKey,
};
use crate::tests::domains::fintech::execution_tier::FintechTier;

use self::topology::factor_signal_aspect;

type FinancialRuntime = SignalRuntime<(), (), (), (), FintechTier>;

use compiled_authority::source_result;
use compiled_authority::CompiledFinancialWorldKind;
pub(crate) use compiled_authority::{
    compile_financial_world, compile_financial_world_with_policy, CompiledFinancialWorld,
};
pub(in crate::tests::domains::fintech) use dependency_rewire::FinancialDependencyRewireEvidence;
pub(in crate::tests::domains::fintech) use factor_sequence::FinancialFactorSequenceEvidence;
pub(in crate::tests::domains::fintech) use lifecycle_composition::FinancialBranchLifecycleCompletion;
pub(in crate::tests::domains::fintech) use locality_execution::strategy_work_projection;
pub(crate) use locality_execution::{
    compile_financial_locality_world, compile_financial_locality_world_at_tier,
    compile_financial_locality_world_with_policy, FinancialPerformanceBatchReport,
    LocalityOptionalObservationInventory,
};
pub(in crate::tests::domains::fintech) use locality_execution::{
    FinancialLocalityRedObservation, FinancialPerformedCanonicalWork, FinancialPerformedWorkOrigin,
    FinancialRestoreLifecycleEvidence,
};
pub(in crate::tests::domains::fintech) use quote_translation::FinancialQuoteTranslationEvidence;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct FactorSourceHandle(
    pub(in crate::tests::domains::fintech) NodeId,
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct PositionSemanticHandles {
    pub(in crate::tests::domains::fintech) valuation: NodeId,
    pub(in crate::tests::domains::fintech) risk: NodeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::tests::domains::fintech) struct ConsumerSemanticHandle(
    pub(in crate::tests::domains::fintech) NodeId,
);

#[derive(Clone, Debug)]
pub(crate) struct FinancialSemanticHandles {
    pub(super) factors: BTreeMap<MarketFactorKey, FactorSourceHandle>,
    pub(super) positions: BTreeMap<InstrumentId, PositionSemanticHandles>,
    pub(super) consumers: BTreeMap<FinancialConsumerRole, ConsumerSemanticHandle>,
}

impl FinancialSemanticHandles {
    pub(in crate::tests::domains::fintech) fn factor(
        &self,
        factor: MarketFactorKey,
    ) -> FactorSourceHandle {
        self.factors[&factor]
    }

    pub(in crate::tests::domains::fintech) fn position(
        &self,
        instrument: InstrumentId,
    ) -> PositionSemanticHandles {
        self.positions[&instrument]
    }

    pub(in crate::tests::domains::fintech) fn consumer(
        &self,
        role: FinancialConsumerRole,
    ) -> ConsumerSemanticHandle {
        self.consumers[&role]
    }

    pub(crate) fn node_for(&self, key: SemanticOutputKey) -> NodeId {
        match key {
            SemanticOutputKey::Factor(factor) => self.factor(factor).0,
            SemanticOutputKey::Valuation(instrument) => self.position(instrument).valuation,
            SemanticOutputKey::Risk(instrument) => self.position(instrument).risk,
            SemanticOutputKey::Consumer(role) => self.consumer(role).0,
        }
    }

    pub(super) fn derived_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.positions
            .values()
            .flat_map(|handles| [handles.valuation, handles.risk])
            .chain(self.consumers.values().map(|handle| handle.0))
    }
}

#[derive(Clone, Default)]
pub(crate) struct FinancialEvaluationLedger {
    counts: Arc<Mutex<BTreeMap<SemanticOutputKey, u64>>>,
}

impl FinancialEvaluationLedger {
    pub(super) fn record(&self, key: SemanticOutputKey) {
        let mut counts = self.counts.lock().expect("financial ledger lock poisoned");
        *counts.entry(key).or_default() += 1;
    }

    pub(super) fn clear(&self) {
        self.counts
            .lock()
            .expect("financial ledger lock poisoned")
            .clear();
    }

    pub(crate) fn observed_work(&self) -> BTreeSet<SemanticOutputKey> {
        self.counts
            .lock()
            .expect("financial ledger lock poisoned")
            .keys()
            .copied()
            .collect()
    }

    pub(in crate::tests::domains::fintech) fn count(&self, key: SemanticOutputKey) -> u64 {
        self.counts
            .lock()
            .expect("financial ledger lock poisoned")
            .get(&key)
            .copied()
            .unwrap_or_default()
    }
}

impl CompiledFinancialWorld {
    pub(crate) fn semantic_output_keys(&self) -> BTreeSet<SemanticOutputKey> {
        self.projection.iter().map(|(key, _)| key).collect()
    }

    pub(crate) fn definition(&self) -> &FinancialWorldDefinition {
        match &self.kind {
            CompiledFinancialWorldKind::Portfolio(portfolio) => &portfolio.definition,
            CompiledFinancialWorldKind::Locality(locality) => locality.definition(),
        }
    }

    pub(crate) fn economic_snapshot(&self) -> &FinancialEconomicSnapshot {
        &self.economic_snapshot
    }

    pub(crate) fn projection(&self) -> &FinancialSemanticProjection {
        &self.projection
    }

    pub(crate) fn handles(&self) -> &FinancialSemanticHandles {
        &self.handles
    }

    pub(crate) fn ledger(&self) -> &FinancialEvaluationLedger {
        &self.ledger
    }

    pub(crate) fn graph(&self) -> &SignalGraph {
        self.runtime.graph()
    }

    pub(super) fn runtime_mut(&mut self) -> &mut FinancialRuntime {
        &mut self.runtime
    }

    pub(in crate::tests::domains::fintech) fn metrics(&self) -> RuntimeMetrics {
        self.runtime.observe().metrics()
    }

    pub(in crate::tests::domains::fintech) fn node_version(
        &self,
        key: SemanticOutputKey,
    ) -> Result<AspectVersion, SignalError> {
        self.runtime
            .graph()
            .node_aspect_version(self.handles.node_for(key))
    }

    pub(crate) fn node_state(&self, key: SemanticOutputKey) -> Result<NodeState, SignalError> {
        self.runtime.graph().get_state(self.handles.node_for(key))
    }

    pub(in crate::tests::domains::fintech) fn factor_slot(
        &self,
        factor: MarketFactorKey,
    ) -> crate::data::aspect::Aspect {
        factor_signal_aspect(&self.definition, factor)
    }
}
