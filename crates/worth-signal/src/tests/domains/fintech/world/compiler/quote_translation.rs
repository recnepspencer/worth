use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;

use super::super::{
    FinancialConsumerRole, FinancialWorldDefinition, InstrumentId, MarketFactorKey,
    SemanticOutputKey,
};
use super::CompiledFinancialWorld;

#[derive(Clone, Copy)]
struct FinancialCauseHop {
    producer: NodeId,
    consumer: NodeId,
    aspect: Aspect,
}

pub(in crate::tests::domains::fintech) struct FinancialQuoteTranslationEvidence {
    source_to_valuation: FinancialCauseHop,
    valuation_to_risk: FinancialCauseHop,
    risk_to_matched: FinancialCauseHop,
    risk_to_unmatched: FinancialCauseHop,
    matched_evaluations: u64,
    unmatched_evaluations: u64,
}

impl FinancialQuoteTranslationEvidence {
    pub(in crate::tests::domains::fintech) fn certifies_price_to_risk_translation(
        &self,
        compiled: &CompiledFinancialWorld,
        factor: MarketFactorKey,
        instrument: InstrumentId,
    ) -> bool {
        let handles = compiled.handles();
        let source = handles.factor(factor).0;
        let position = handles.position(instrument);
        let matched = handles.consumer(FinancialConsumerRole::RiskMatched).0;
        let unmatched = handles.consumer(FinancialConsumerRole::RiskUnmatched).0;
        let price = super::super::super::aspects::PRICE;
        let risk = super::super::super::aspects::RISK;
        self.source_to_valuation
            .matches(source, position.valuation, price)
            && self
                .valuation_to_risk
                .matches(position.valuation, position.risk, price)
            && self.risk_to_matched.matches(position.risk, matched, risk)
            && self
                .risk_to_unmatched
                .matches(position.risk, unmatched, risk)
            && self.matched_evaluations == 1
            && self.unmatched_evaluations == 0
    }
}

impl FinancialCauseHop {
    fn from_single(causes: &[ResolvedDependencyCause]) -> Result<Self, SignalError> {
        let [cause] = causes else {
            return Err(SignalError::internal(
                "quote translation step did not expose exactly one dependency cause",
            ));
        };
        Ok(Self {
            producer: cause.binding_axes.producer,
            consumer: cause.binding_axes.consumer,
            aspect: cause.binding_axes.aspect,
        })
    }

    fn matches(&self, producer: NodeId, consumer: NodeId, aspect: Aspect) -> bool {
        self.producer == producer && self.consumer == consumer && self.aspect == aspect
    }
}

impl CompiledFinancialWorld {
    pub(in crate::tests::domains::fintech) fn apply_quote_translation_change(
        &mut self,
        next_definition: FinancialWorldDefinition,
        factor: MarketFactorKey,
        instrument: InstrumentId,
    ) -> Result<FinancialQuoteTranslationEvidence, SignalError> {
        // This standalone caller declares the operational serial memory policy.
        let serial_request = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(
                crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
            ),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

        self.stage_factor_change(next_definition, factor)?;
        let position = self.handles.position(instrument);
        let matched = self.handles.consumer(FinancialConsumerRole::RiskMatched).0;
        let unmatched = self
            .handles
            .consumer(FinancialConsumerRole::RiskUnmatched)
            .0;
        let source_to_valuation = FinancialCauseHop::from_single(
            self.runtime.graph().pending_causes(position.valuation)?,
        )?;
        let program = self.program();
        let evaluator = program.evaluator();

        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.read(position.valuation, &evaluator)?;
            Ok(())
        })?;
        let valuation_to_risk =
            FinancialCauseHop::from_single(self.runtime.graph().pending_causes(position.risk)?)?;

        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.read(position.risk, &evaluator)?;
            Ok(())
        })?;
        let risk_to_matched =
            FinancialCauseHop::from_single(self.runtime.graph().pending_causes(matched)?)?;
        let risk_to_unmatched =
            FinancialCauseHop::from_single(self.runtime.graph().pending_causes(unmatched)?)?;

        self.runtime.transaction(request_execution, &mut (), |tx| {
            tx.read(matched, &evaluator)?;
            tx.read(unmatched, &evaluator)?;
            Ok(())
        })?;
        Ok(FinancialQuoteTranslationEvidence {
            source_to_valuation,
            valuation_to_risk,
            risk_to_matched,
            risk_to_unmatched,
            matched_evaluations: self.ledger.count(SemanticOutputKey::Consumer(
                FinancialConsumerRole::RiskMatched,
            )),
            unmatched_evaluations: self.ledger.count(SemanticOutputKey::Consumer(
                FinancialConsumerRole::RiskUnmatched,
            )),
        })
    }
}
