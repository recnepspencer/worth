//! One registered product's exact observer for a single receipt comparison.
use std::{collections::BTreeMap, sync::Arc};
use worth_relational::facade::snapshots::SnapshotHandle;

use super::{provider_rejected, snapshot_read_set_failure, WorthQueryPrimaryGraphProvider};
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    WorthQueryPrimaryGraphApplicationDecisionFact,
};
use crate::domain_computation::provider_session::WorthQuerySessionReadAuthority;
use crate::domain_computation::{
    WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidenceView,
    WorthQueryDecisionReadSetFailure, WorthQueryDecisionReadSetFreshnessOutcome,
    WorthQueryFreshDecisionReadSet, WorthQueryProviderSessionView,
};

mod indexed;
use indexed::PreparedSelections;

struct PrimaryDecisionReadSetComparison<'provider> {
    provider: &'provider WorthQueryPrimaryGraphProvider,
    facts: Arc<BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact>>,
    snapshot: Option<SnapshotHandle>,
    selections: PreparedSelections,
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn compare_application_read_set(
        &self,
        authority: &WorthQuerySessionReadAuthority<'_>,
        receipt: WorthQueryCompleteDecisionReadSetReceipt,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        authority.validate_decision_read_set_receipt(&receipt)?;
        let mut comparison =
            PrimaryDecisionReadSetComparison::acquire(self, authority.session(), control)?;
        authority.compare_decision_read_set_with(receipt, |evidence, admission| {
            comparison.compare(authority.session(), evidence, admission, control)
        })
    }

    pub(in crate::domain_computation::primary_graph) fn recompare_application_read_set(
        &self,
        authority: &WorthQuerySessionReadAuthority<'_>,
        fresh: WorthQueryFreshDecisionReadSet,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        authority.validate_fresh_decision_read_set(&fresh)?;
        let mut comparison =
            PrimaryDecisionReadSetComparison::acquire(self, authority.session(), control)?;
        authority.recompare_fresh_decision_read_set_with(fresh, |evidence, admission| {
            comparison.compare(authority.session(), evidence, admission, control)
        })
    }
}

impl<'provider> PrimaryDecisionReadSetComparison<'provider> {
    fn acquire(
        provider: &'provider WorthQueryPrimaryGraphProvider,
        session: WorthQueryProviderSessionView<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        check_control(control)?;
        let (facts, product) = provider
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .shared_observed_facts(session)
            .ok_or_else(provider_rejected)?;
        let snapshot = provider.graph.with_runtime_mut(|runtime| {
            crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, product.observation().basis().relational_basis(),
            )
        }).map_err(|denial| snapshot_read_set_failure(denial.into()))?;
        let comparison = Self {
            provider,
            facts,
            snapshot: Some(snapshot),
            selections: PreparedSelections::new(),
        };
        check_control(control)?;
        Ok(comparison)
    }

    fn compare(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        check_control(control)?;
        // Retaining the map is custody, not permission to bypass live registration.
        // Release this mutex before entering the runtime (the existing lock order).
        let still_registered = self
            .provider
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retains_observed_facts(session, &self.facts);
        if !still_registered {
            return Err(provider_rejected());
        }
        let fact = self
            .facts
            .get(evidence.locator().identity())
            .ok_or_else(provider_rejected)?;
        let snapshot = self
            .snapshot
            .as_ref()
            .expect("comparison owns its exact observer");
        let fresh = self.provider.graph.with_runtime(|runtime| match fact {
            WorthQueryPrimaryGraphApplicationDecisionFact::Application { fact, .. }
            | WorthQueryPrimaryGraphApplicationDecisionFact::ObservedSource { fact }
            | WorthQueryPrimaryGraphApplicationDecisionFact::ApplicationObservedSource {
                fact,
                ..
            } => self
                .selections
                .remains_equal(fact, runtime, snapshot, control),
            _ => Ok(fact.remains_equal_in(runtime, snapshot)),
        })?;
        check_control(control)?;
        admission.observe_current_version(if fresh {
            evidence.physical_version_evidence().to_owned()
        } else {
            format!("application-stale:{}", evidence.locator().identity())
        })
    }
}

impl Drop for PrimaryDecisionReadSetComparison<'_> {
    fn drop(&mut self) {
        // Prepared tokens retain the root; release them before the active observer.
        self.selections.clear();
        if let Some(snapshot) = self.snapshot.take() {
            self.provider.graph.with_runtime_mut(|runtime| {
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            });
        }
    }
}

fn check_control(control: StorageControl<'_, '_>) -> Result<(), WorthQueryDecisionReadSetFailure> {
    if let Some(stop) = control.request().and_then(|request| request.interruption()) {
        return Err(WorthQueryDecisionReadSetFailure::new(
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RequestInterrupted(
                stop,
            ),
            "decision read-set authority denied",
        ));
    }
    control
        .policy()
        .check_live()
        .map_err(WorthQueryDecisionReadSetFailure::allocation_denied)
}

#[cfg(test)]
mod tests;
