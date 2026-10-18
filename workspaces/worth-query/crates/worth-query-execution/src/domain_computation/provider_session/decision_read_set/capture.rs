use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use super::{
    WorthQueryDecisionFactAdmission, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidence,
    WorthQueryDecisionFactRequest, WorthQueryDecisionFactRequestView,
    WorthQueryDecisionReadSetDenialKind, WorthQueryDecisionReadSetFailure,
};
use crate::domain_computation::provider_session::WorthQuerySessionReadAuthority;
use worth_execution::{ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder};
use worth_query_installation::facade::WorthQueryDecisionFactCardinality;
mod primary;
mod request_admission;
use request_admission::admit_requests;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryDecisionReadSetCounters {
    requested_facts: usize,
    provider_calls: usize,
    compared_facts: usize,
    stale_facts: usize,
    false_conflicts: usize,
}

impl WorthQueryDecisionReadSetCounters {
    pub fn requested_facts(self) -> usize {
        self.requested_facts
    }

    pub fn provider_calls(self) -> usize {
        self.provider_calls
    }

    pub fn compared_facts(self) -> usize {
        self.compared_facts
    }

    pub fn stale_facts(self) -> usize {
        self.stale_facts
    }

    pub fn false_conflicts(self) -> usize {
        self.false_conflicts
    }
}

pub struct WorthQueryCompleteDecisionReadSetReceipt {
    identity: Arc<str>,
    session_binding_identity: Arc<str>,
    evidence: Arc<ExecutionArray<WorthQueryDecisionFactEvidence>>,
    counters: WorthQueryDecisionReadSetCounters,
    request: Option<WorthQueryRequestScope>,
    primary: Option<crate::domain_computation::primary_graph::RegisteredPrimaryRead>,
    _primary_custody: Option<crate::domain_computation::primary_graph::RegisteredPrimaryRead>,
    _primary_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryCompleteDecisionReadSetReceipt {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn fact_count(&self) -> usize {
        self.evidence.len()
            + self
                .primary
                .as_ref()
                .map_or(0, |read| read.facts().grouped_count())
    }

    pub fn counters(&self) -> WorthQueryDecisionReadSetCounters {
        self.counters
    }
}

pub enum WorthQueryDecisionReadSetFreshnessOutcome {
    Fresh(WorthQueryFreshDecisionReadSet),
    Stale(WorthQueryStaleDecisionReadSet),
}

pub struct WorthQueryFreshDecisionReadSet {
    receipt: WorthQueryCompleteDecisionReadSetReceipt,
    counters: WorthQueryDecisionReadSetCounters,
}

impl WorthQueryFreshDecisionReadSet {
    pub fn read_set_identity(&self) -> &str {
        self.receipt.identity()
    }

    pub fn counters(&self) -> WorthQueryDecisionReadSetCounters {
        self.counters
    }

    pub(crate) fn belongs_to(&self, binding_identity: &str) -> bool {
        self.receipt.session_binding_identity.as_ref() == binding_identity
    }

    pub(crate) fn contains_locator(&self, identity: &str) -> bool {
        if let Some(table) = &self.receipt.primary {
            return table.facts().contains_key(identity);
        }
        self.receipt
            .evidence
            .iter()
            .any(|evidence| evidence.view().locator().identity() == identity)
    }
}

#[derive(Debug)]
pub struct WorthQueryStaleDecisionReadSet {
    read_set_identity: Arc<str>,
    stale_evidence_identities: Arc<[Arc<str>]>,
    counters: WorthQueryDecisionReadSetCounters,
}

impl WorthQueryStaleDecisionReadSet {
    pub fn read_set_identity(&self) -> &str {
        &self.read_set_identity
    }

    pub fn stale_fact_count(&self) -> usize {
        self.stale_evidence_identities.len()
    }

    pub fn counters(&self) -> WorthQueryDecisionReadSetCounters {
        self.counters
    }

    #[cfg(test)]
    pub(crate) fn stale_fact_identities_for_test(&self) -> &[Arc<str>] {
        &self.stale_evidence_identities
    }
}

impl WorthQuerySessionReadAuthority<'_> {
    pub fn capture_decision_read_set(
        &self,
        requests: impl IntoIterator<Item = WorthQueryDecisionFactRequest>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
        request: Option<&WorthQueryRequestScope>,
    ) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure> {
        let binding = self.binding();
        let (requests, mut counters) = admit_requests(self, requests, request)?;
        let mut evidence = ExecutionArrayBuilder::allocate(requests.len(), allocation_policy)
            .map_err(WorthQueryDecisionReadSetFailure::allocation_denied)?;
        for requested in requests {
            check_request(request)?;
            evidence
                .check_live()
                .map_err(WorthQueryDecisionReadSetFailure::allocation_denied)?;
            counters.provider_calls += 1;
            let admission = WorthQueryDecisionFactAdmission::new(requested.clone(), binding);
            let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.provider().observe_decision_fact(
                    self.session(),
                    WorthQueryDecisionFactRequestView::new(&requested),
                    admission,
                )
            }));
            check_request(request)?;
            let fact = provider_result(invocation)?;
            if !fact.belongs_to(binding, &requested) {
                return Err(denial(
                    WorthQueryDecisionReadSetDenialKind::EvidenceSubstitution,
                ));
            }
            evidence
                .push(fact)
                .map_err(WorthQueryDecisionReadSetFailure::allocation_denied)?;
        }
        let evidence = Arc::new(
            evidence
                .seal()
                .map_err(WorthQueryDecisionReadSetFailure::allocation_denied)?,
        );
        Ok(WorthQueryCompleteDecisionReadSetReceipt {
            identity: binding.retain_canonical_identity(),
            session_binding_identity: binding.retain_canonical_identity(),
            evidence,
            counters,
            request: request.cloned(),
            primary: None,
            _primary_custody: None,
            _primary_memory: None,
        })
    }

    pub fn compare_decision_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: WorthQueryCompleteDecisionReadSetReceipt,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        self.compare_decision_read_set_with(receipt, |evidence, admission| {
            self.provider()
                .compare_decision_fact(execution, self.session(), evidence, admission)
        })
    }

    pub(crate) fn compare_decision_read_set_with(
        &self,
        receipt: WorthQueryCompleteDecisionReadSetReceipt,
        mut compare: impl FnMut(
            super::WorthQueryDecisionFactEvidenceView<'_>,
            WorthQueryDecisionFactComparisonAdmission,
        ) -> Result<
            WorthQueryDecisionFactComparisonEvidence,
            WorthQueryDecisionReadSetFailure,
        >,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        self.validate_decision_read_set_receipt(&receipt)?;
        if receipt.primary.is_some() {
            return Err(denial(
                WorthQueryDecisionReadSetDenialKind::EvidenceSubstitution,
            ));
        }
        let mut counters = receipt.counters;
        let mut stale = Vec::new();
        for evidence in receipt.evidence.iter() {
            check_request(receipt.request.as_ref())?;
            counters.provider_calls += 1;
            counters.compared_facts += 1;
            let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                compare(
                    evidence.view(),
                    WorthQueryDecisionFactComparisonAdmission::new(evidence),
                )
            }));
            check_request(receipt.request.as_ref())?;
            let comparison = provider_comparison(invocation)?;
            if !comparison.belongs_to(evidence) {
                return Err(denial(
                    WorthQueryDecisionReadSetDenialKind::EvidenceSubstitution,
                ));
            }
            if !comparison.is_fresh() {
                counters.stale_facts += 1;
                stale.push(Arc::<str>::from(evidence.identity()));
            }
        }
        if stale.is_empty() {
            Ok(WorthQueryDecisionReadSetFreshnessOutcome::Fresh(
                WorthQueryFreshDecisionReadSet { receipt, counters },
            ))
        } else {
            Ok(WorthQueryDecisionReadSetFreshnessOutcome::Stale(
                WorthQueryStaleDecisionReadSet {
                    read_set_identity: receipt.identity,
                    stale_evidence_identities: stale.into(),
                    counters,
                },
            ))
        }
    }

    pub(crate) fn validate_decision_read_set_receipt(
        &self,
        receipt: &WorthQueryCompleteDecisionReadSetReceipt,
    ) -> Result<(), WorthQueryDecisionReadSetFailure> {
        if receipt.session_binding_identity.as_ref() != self.binding().canonical_identity() {
            return Err(denial(
                WorthQueryDecisionReadSetDenialKind::EvidenceSubstitution,
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_fresh_decision_read_set(
        &self,
        fresh: &WorthQueryFreshDecisionReadSet,
    ) -> Result<(), WorthQueryDecisionReadSetFailure> {
        self.validate_decision_read_set_receipt(&fresh.receipt)
    }

    pub(crate) fn recompare_fresh_decision_read_set_with(
        &self,
        fresh: WorthQueryFreshDecisionReadSet,
        compare: impl FnMut(
            super::WorthQueryDecisionFactEvidenceView<'_>,
            WorthQueryDecisionFactComparisonAdmission,
        ) -> Result<
            WorthQueryDecisionFactComparisonEvidence,
            WorthQueryDecisionReadSetFailure,
        >,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        self.compare_decision_read_set_with(fresh.receipt, compare)
    }

    pub(crate) fn recompare_fresh_decision_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh: WorthQueryFreshDecisionReadSet,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        self.compare_decision_read_set(execution, fresh.receipt)
    }
}

fn provider_result(
    result: Result<
        Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure>,
        Box<dyn std::any::Any + Send>,
    >,
) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
    match result {
        Ok(result) => result,
        Err(_) => Err(denial(
            WorthQueryDecisionReadSetDenialKind::ProviderPanicked,
        )),
    }
}

fn provider_comparison(
    result: Result<
        Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure>,
        Box<dyn std::any::Any + Send>,
    >,
) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
    match result {
        Ok(result) => result,
        Err(_) => Err(denial(
            WorthQueryDecisionReadSetDenialKind::ProviderPanicked,
        )),
    }
}

fn denial(kind: WorthQueryDecisionReadSetDenialKind) -> WorthQueryDecisionReadSetFailure {
    WorthQueryDecisionReadSetFailure::new(kind, "decision read-set authority denied")
}

fn check_request(
    request: Option<&WorthQueryRequestScope>,
) -> Result<(), WorthQueryDecisionReadSetFailure> {
    match request.and_then(WorthQueryRequestScope::interruption) {
        Some(stop) => Err(denial(
            WorthQueryDecisionReadSetDenialKind::RequestInterrupted(stop),
        )),
        None => Ok(()),
    }
}
