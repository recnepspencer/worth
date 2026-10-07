use std::collections::BTreeSet;

use crate::data::error::SignalError;
use crate::tests::domains::fintech::invalidation::ComparatorScenarioOutcome;
use crate::tests::domains::fintech::world::{
    CompiledFinancialWorld, FinancialBranchLifecycleCompletion, FinancialConsumerRole,
    FinancialDependencyRewireEvidence, FinancialFactorSequenceEvidence,
    FinancialQuoteTranslationEvidence, FinancialScenarioIdentity, InstrumentId, MarketFactorKey,
    SemanticOutputKey,
};

#[derive(Clone)]
pub(in crate::tests::domains::fintech) struct FinancialScenarioCompletion {
    kind: FinancialScenarioCompletionKind,
}

#[derive(Clone)]
struct ScenarioCompletionSeal;

#[derive(Clone)]
enum FinancialScenarioCompletionKind {
    Ordinary {
        scenario: FinancialScenarioIdentity,
        _seal: ScenarioCompletionSeal,
    },
    Branch(FinancialBranchLifecycleCompletion),
}

impl FinancialScenarioCompletion {
    fn ordinary(scenario: FinancialScenarioIdentity) -> Self {
        Self {
            kind: FinancialScenarioCompletionKind::Ordinary {
                scenario,
                _seal: ScenarioCompletionSeal,
            },
        }
    }

    pub(super) fn quote_translation(
        compiled: &CompiledFinancialWorld,
        evidence: &FinancialQuoteTranslationEvidence,
        factor: MarketFactorKey,
        instrument: InstrumentId,
    ) -> Result<Self, SignalError> {
        require(
            evidence.certifies_price_to_risk_translation(compiled, factor, instrument),
            "quote translation evidence lacks the exact PRICE-to-RISK cause chain",
        )?;
        Ok(Self::ordinary(
            FinancialScenarioIdentity::QuoteToRiskAspectTranslation,
        ))
    }

    pub(super) fn heterogeneous(outcome: &ComparatorScenarioOutcome) -> Result<Self, SignalError> {
        require(
            outcome.final_revision > outcome.baseline_revision
                && outcome.observed_work == outcome.required_work,
            "heterogeneous comparator evidence is incomplete",
        )?;
        Ok(Self::ordinary(
            FinancialScenarioIdentity::HeterogeneousConsumerComparators,
        ))
    }

    pub(super) fn tolerance(
        small: &ComparatorScenarioOutcome,
        large: &ComparatorScenarioOutcome,
    ) -> Result<Self, SignalError> {
        require(
            small.observed_work == BTreeSet::from([SemanticOutputKey::Factor(small.factor)])
                && large.observed_work == large.required_work
                && large.final_revision > small.final_revision,
            "producer tolerance evidence lacks suppressed and admitted twins",
        )?;
        Ok(Self::ordinary(
            FinancialScenarioIdentity::ToleranceSuppressedRepricing,
        ))
    }

    pub(super) fn producer_slots(
        compiled: &CompiledFinancialWorld,
        changed: MarketFactorKey,
        unrelated: MarketFactorKey,
    ) -> Result<Self, SignalError> {
        require(
            compiled.factor_slot(changed) == compiled.factor_slot(unrelated)
                && compiled.handles().factor(changed).0 != compiled.handles().factor(unrelated).0
                && !compiled
                    .ledger()
                    .observed_work()
                    .contains(&SemanticOutputKey::Factor(unrelated))
                && compiled.node_version(SemanticOutputKey::Factor(unrelated))?
                    == *compiled.baseline_node_version(SemanticOutputKey::Factor(unrelated)),
            "producer-local slot collision evidence is incomplete",
        )?;
        Ok(Self::ordinary(
            FinancialScenarioIdentity::ProducerLocalFactorSlotCollision,
        ))
    }

    pub(super) fn partition(
        evidence: &FinancialFactorSequenceEvidence,
    ) -> Result<Self, SignalError> {
        let details = evidence
            .pending_scopes()
            .iter()
            .filter_map(|scope| scope.path().segments().get(1).map(String::as_str))
            .collect::<BTreeSet<_>>();
        require(
            evidence.gated_consumer_was_pending()
                && details == BTreeSet::from(["usd-1y", "eur-1y"]),
            "partition-union evidence is incomplete",
        )?;
        Ok(Self::ordinary(
            FinancialScenarioIdentity::PartitionedCurveBucketBump,
        ))
    }

    pub(super) fn gated(
        small: &CompiledFinancialWorld,
        large: &CompiledFinancialWorld,
    ) -> Result<Self, SignalError> {
        let key = SemanticOutputKey::Consumer(FinancialConsumerRole::RiskThreshold);
        require(
            !small.ledger().observed_work().contains(&key)
                && large.ledger().observed_work().contains(&key),
            "gated repricing evidence lacks blocked and released twins",
        )?;
        Ok(Self::ordinary(
            FinancialScenarioIdentity::GatedRepricingRelease,
        ))
    }

    pub(super) fn rewire(
        evidence: &FinancialDependencyRewireEvidence,
    ) -> Result<Self, SignalError> {
        require(
            evidence.stale_cause_rejected()
                && evidence.cycle_rejected()
                && evidence.topology_owner()
                    == SemanticOutputKey::Valuation(InstrumentId("EURUSD-1Y-FWD"))
                && evidence.stale_revision() == evidence.baseline_revision()
                && evidence.final_revision() == evidence.baseline_revision() + 3,
            "dependency-rewire evidence is incomplete",
        )?;
        Ok(Self::ordinary(
            FinancialScenarioIdentity::InstrumentDependencyRewire,
        ))
    }

    pub(super) fn branch(completion: FinancialBranchLifecycleCompletion) -> Self {
        Self {
            kind: FinancialScenarioCompletionKind::Branch(completion),
        }
    }

    pub(super) fn scenario(&self) -> FinancialScenarioIdentity {
        match &self.kind {
            FinancialScenarioCompletionKind::Ordinary { scenario, .. } => *scenario,
            FinancialScenarioCompletionKind::Branch(_) => {
                FinancialScenarioIdentity::BranchShockRestoreReplay
            }
        }
    }
}

fn require(condition: bool, message: &'static str) -> Result<(), SignalError> {
    condition
        .then_some(())
        .ok_or_else(|| SignalError::internal(message))
}
