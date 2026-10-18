//! One registered product's exact observer for a single receipt comparison.
use std::sync::Arc;
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

pub(in crate::domain_computation) struct PrimaryDecisionReadSetComparison<'provider> {
    provider: &'provider WorthQueryPrimaryGraphProvider,
    facts: Arc<crate::domain_computation::primary_graph::ApplicationFactTable>,
    registration: crate::domain_computation::primary_graph::RegisteredPrimaryRead,
    snapshot: Option<SnapshotHandle>,
    selections: PreparedSelections,
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn capture_application_read_set(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure> {
        check_control(control)?;
        let read = self
            .attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .registered_read(authority.session())
            .ok_or_else(provider_rejected)?;
        let receipt = authority.capture_primary_read_set(read.clone(), control)?;
        let registered = self
            .attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retains_registered_read(authority.session(), &read);
        if !registered {
            return Err(provider_rejected());
        }
        check_control(control)?;
        Ok(receipt)
    }
    pub(in crate::domain_computation::primary_graph) fn compare_application_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        receipt: WorthQueryCompleteDecisionReadSetReceipt,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        authority.validate_decision_read_set_receipt(&receipt)?;
        let mut comparison = PrimaryDecisionReadSetComparison::acquire(
            execution,
            self,
            authority.session(),
            control,
        )?;
        authority.compare_primary_read_set(execution, receipt, &mut comparison, control)
    }

    pub(in crate::domain_computation::primary_graph) fn recompare_application_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        fresh: WorthQueryFreshDecisionReadSet,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        authority.validate_fresh_decision_read_set(&fresh)?;
        let mut comparison = PrimaryDecisionReadSetComparison::acquire(
            execution,
            self,
            authority.session(),
            control,
        )?;
        authority.recompare_primary_read_set(execution, fresh, &mut comparison, control)
    }
}

impl<'provider> PrimaryDecisionReadSetComparison<'provider> {
    pub(in crate::domain_computation) fn validate_registration(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        captured: Option<&crate::domain_computation::primary_graph::RegisteredPrimaryRead>,
        control: StorageControl<'_, '_>,
    ) -> Result<(), WorthQueryDecisionReadSetFailure> {
        check_control(control)?;
        let store = self
            .provider
            .attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let own = store.retains_registered_read(session, &self.registration);
        let captured = captured.is_none_or(|read| store.retains_registered_read(session, read));
        drop(store);
        if !own || !captured {
            return Err(provider_rejected());
        }
        check_control(control)
    }

    pub(in crate::domain_computation) fn closes_original(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        read: &crate::domain_computation::primary_graph::RegisteredPrimaryRead,
        control: StorageControl<'_, '_>,
    ) -> Result<bool, WorthQueryDecisionReadSetFailure> {
        self.validate_registration(session, Some(read), control)?;
        let table = read.facts();
        let Some((_, original)) = table.closed_prefix() else {
            return Ok(false);
        };
        if !self
            .provider
            .attempts
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .binds_original_read(session, original)
        {
            return Ok(false);
        }
        let snapshot = self.snapshot.as_ref().expect("genuine comparison observer");
        let same = self
            .provider
            .graph
            .with_runtime(execution, |runtime| {
                original.matches_source(runtime, snapshot, control)
            })
            .map_err(super::storage_read_set_failure)?;
        check_control(control)?;
        Ok(same)
    }
    fn acquire(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &'provider WorthQueryPrimaryGraphProvider,
        session: WorthQueryProviderSessionView<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, WorthQueryDecisionReadSetFailure> {
        check_control(control)?;
        let (registration, product) = {
            let store = provider.attempts.lock().unwrap_or_else(|p| p.into_inner());
            let read = store
                .registered_read(session)
                .ok_or_else(provider_rejected)?;
            let (_, product) = store
                .shared_observed_facts(session)
                .ok_or_else(provider_rejected)?;
            (read, product)
        };
        let facts = registration.facts().clone();
        let snapshot = provider.graph.with_runtime_mut(execution, |runtime| {
            crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, product.observation().basis().relational_basis(),
            )
        }).map_err(|denial| snapshot_read_set_failure(denial.into()))?;
        let comparison = Self {
            provider,
            facts,
            registration,
            snapshot: Some(snapshot),
            selections: PreparedSelections::new(),
        };
        check_control(control)?;
        Ok(comparison)
    }

    pub(in crate::domain_computation) fn compare(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        check_control(control)?;
        // Retaining the map is custody, not permission to bypass live registration.
        // Release this mutex before entering the runtime (the existing lock order).
        self.validate_registration(session, None, control)?;
        let fact = self
            .facts
            .get(evidence.locator().identity())
            .ok_or_else(provider_rejected)?;
        let snapshot = self
            .snapshot
            .as_ref()
            .expect("comparison owns its exact observer");
        let fresh = self
            .provider
            .graph
            .with_runtime(execution, |runtime| match &fact {
                WorthQueryPrimaryGraphApplicationDecisionFact::Application { fact, .. }
                | WorthQueryPrimaryGraphApplicationDecisionFact::ObservedSource { fact }
                | WorthQueryPrimaryGraphApplicationDecisionFact::ApplicationObservedSource {
                    fact,
                    ..
                } => self
                    .selections
                    .remains_equal(execution, fact, runtime, snapshot, control),
                _ => fact
                    .remains_equal_in(execution, runtime, snapshot)
                    .map_err(super::fact_read_set_failure),
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
        self.selections.clear();
        if let Some(snapshot) = self.snapshot.take() {
            self.provider.graph.release_query_snapshot(&snapshot);
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
