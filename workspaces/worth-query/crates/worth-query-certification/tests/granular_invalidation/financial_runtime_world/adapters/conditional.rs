use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use worth_query_host::facade::domain;

use super::super::contract::{
    CurveRiskNode, PortfolioRiskNode, PortfolioSiblingRiskNode, QuoteRiskNode,
};

pub struct FinancialPredicate {
    eligible: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct FinancialGateController(Arc<AtomicBool>);

#[derive(Clone)]
pub struct FinancialQuoteOutputState(Arc<std::sync::atomic::AtomicU64>);

impl FinancialQuoteOutputState {
    pub fn new(value: u64) -> Self {
        Self(Arc::new(std::sync::atomic::AtomicU64::new(value)))
    }

    pub fn set(&self, value: u64) {
        self.0.store(value, Ordering::Release);
    }
}

pub struct QuoteOutputVersionProvider(pub FinancialQuoteOutputState);

impl domain::WorthQueryHostConditionalOutputVersionProvider<QuoteRiskNode>
    for QuoteOutputVersionProvider
{
    fn semantic_identity(&self) -> &'static str {
        "worth.query.financial.quote-output-version"
    }

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        Ok(domain::WorthQueryHostProviderHeapRetention::try_from_parts(
            [
                domain::WorthQueryHostProviderHeapRetention::arc_allocation_bytes(
                    self.0 .0.as_ref(),
                ),
            ],
        )?)
    }

    fn output_version(
        &self,
        _fallback_attempt: u64,
    ) -> Result<u64, domain::WorthQueryHostPredicateFailure> {
        Ok(self.0 .0.load(Ordering::Acquire))
    }
}

pub struct QuoteToleranceComparator;

impl domain::WorthQueryHostConditionalOutputComparatorProvider<QuoteRiskNode>
    for QuoteToleranceComparator
{
    fn semantic_identity(&self) -> &'static str {
        "worth.query.financial.quote-tolerance-5"
    }

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        Ok(domain::WorthQueryHostProviderHeapRetention::none())
    }

    fn has_meaningful_change(
        &self,
        cached: u64,
        current: u64,
    ) -> Result<bool, domain::WorthQueryHostPredicateFailure> {
        Ok(cached.abs_diff(current) > 5)
    }
}

impl FinancialPredicate {
    pub fn blocked() -> (Self, FinancialGateController) {
        let eligible = Arc::new(AtomicBool::new(false));
        (
            Self {
                eligible: Arc::clone(&eligible),
            },
            FinancialGateController(eligible),
        )
    }
}

impl FinancialGateController {
    pub fn release(&self) {
        self.0.store(true, Ordering::Release);
    }
}

macro_rules! eligible_predicate {
    ($node:ty, $identity:literal) => {
        impl domain::WorthQueryHostConditionalPredicateProvider<$node> for FinancialPredicate {
            const SEMANTIC_IDENTITY: &'static str = $identity;

            fn retained_heap_bytes(
                &self,
            ) -> Result<
                domain::WorthQueryHostProviderHeapRetention,
                domain::WorthQueryHostProviderRetentionOverflow,
            > {
                Ok(domain::WorthQueryHostProviderHeapRetention::try_from_parts(
                    [
                        domain::WorthQueryHostProviderHeapRetention::arc_allocation_bytes(
                            self.eligible.as_ref(),
                        ),
                    ],
                )?)
            }

            fn evaluate(
                &self,
                observation: domain::WorthQueryConditionalObservationView<'_>,
            ) -> Result<
                domain::WorthQueryHostPredicateDecision,
                domain::WorthQueryHostPredicateFailure,
            > {
                Ok(
                    if self.eligible.load(Ordering::Acquire)
                        && observation.dependency(0).is_some_and(|dependency| {
                            matches!(
                                dependency.current(),
                                domain::WorthQueryConditionalObservedValue::Present(_)
                            )
                        })
                    {
                        domain::WorthQueryHostPredicateDecision::Satisfied
                    } else {
                        domain::WorthQueryHostPredicateDecision::Unsatisfied
                    },
                )
            }
        }
    };
}

eligible_predicate!(CurveRiskNode, "worth.query.financial.curve-predicate");
eligible_predicate!(QuoteRiskNode, "worth.query.financial.quote-predicate");
eligible_predicate!(
    PortfolioRiskNode,
    "worth.query.financial.portfolio-predicate"
);
eligible_predicate!(
    PortfolioSiblingRiskNode,
    "worth.query.financial.portfolio-sibling-predicate"
);
