//! Fresh session admission of the primary owner's completed original prefix.
use super::*;
use crate::domain_computation::primary_graph::{
    PrimaryDecisionReadSetComparison, RegisteredPrimaryRead, StorageControl,
};
use worth_query_installation::facade::{
    WorthQueryDecisionFactKind, APPLICATION_DECISION_FACT_FAMILY,
};

impl WorthQuerySessionReadAuthority<'_> {
    fn capture_primary_full(
        &self,
        read: RegisteredPrimaryRead,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure> {
        let table = read.facts();
        let bytes = table
            .full_capture_bytes()
            .ok_or_else(|| denial(WorthQueryDecisionReadSetDenialKind::FactCountOverflow))?;
        let memory = control.policy().reserve_memory(bytes);
        check_request(control.request())?;
        let memory = memory.map_err(WorthQueryDecisionReadSetFailure::allocation_denied)?;
        let mut receipt =
            self.capture_decision_read_set(table.requests(), control.policy(), control.request())?;
        receipt._primary_custody = Some(read);
        receipt._primary_memory = memory;
        Ok(receipt)
    }
    pub(in crate::domain_computation) fn capture_primary_read_set(
        &self,
        read: RegisteredPrimaryRead,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure> {
        let table = read.facts();
        let Some(_) = table.closed_prefix() else {
            return self.capture_primary_full(read, control);
        };
        let family = self
            .plan()
            .decision_fact_families()
            .iter()
            .find(|family| family.identity() == APPLICATION_DECISION_FACT_FAMILY)
            .ok_or_else(|| denial(WorthQueryDecisionReadSetDenialKind::UndeclaredFamily))?;
        if family.kind() != &WorthQueryDecisionFactKind::DomainStructuralProof
            || family.cardinality() != WorthQueryDecisionFactCardinality::Variable
        {
            return self.capture_primary_full(read, control);
        }
        let mut receipt = self.capture_decision_read_set(
            table.explicit_requests(),
            control.policy(),
            control.request(),
        )?;
        receipt.counters.requested_facts = receipt
            .counters
            .requested_facts
            .checked_add(table.grouped_count())
            .ok_or_else(|| denial(WorthQueryDecisionReadSetDenialKind::FactCountOverflow))?;
        receipt._primary_custody = Some(read.clone());
        receipt.primary = Some(read);
        Ok(receipt)
    }

    pub(in crate::domain_computation) fn compare_primary_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut receipt: WorthQueryCompleteDecisionReadSetReceipt,
        comparison: &mut PrimaryDecisionReadSetComparison<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        self.validate_decision_read_set_receipt(&receipt)?;
        let binding = receipt._primary_custody.clone();
        comparison.validate_registration(self.session(), binding.as_ref(), control)?;
        let primary = receipt.primary.take();
        if let Some(table) = &primary
            && !comparison.closes_original(execution, self.session(), table, control)?
        {
            // No authority is inherited across source readmission. Capture
            // and compare all ORIGINAL values under this same fresh session.
            let full = self.capture_primary_full(table.clone(), control)?;
            let outcome = self.compare_decision_read_set_with(full, |evidence, admission| {
                comparison.compare(execution, self.session(), evidence, admission, control)
            })?;
            comparison.validate_registration(self.session(), binding.as_ref(), control)?;
            return Ok(outcome);
        }
        let mut outcome = self.compare_decision_read_set_with(receipt, |evidence, admission| {
            comparison.compare(execution, self.session(), evidence, admission, control)
        })?;
        if let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) = &mut outcome {
            if let Some(table) = &primary
                && !comparison.closes_original(execution, self.session(), table, control)?
            {
                return Err(denial(
                    WorthQueryDecisionReadSetDenialKind::EvidenceSubstitution,
                ));
            }
            fresh.receipt.primary = primary;
        }
        comparison.validate_registration(self.session(), binding.as_ref(), control)?;
        Ok(outcome)
    }
    pub(in crate::domain_computation) fn recompare_primary_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh: WorthQueryFreshDecisionReadSet,
        comparison: &mut PrimaryDecisionReadSetComparison<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        if fresh.receipt.primary.is_none() {
            let binding = fresh.receipt._primary_custody.clone();
            comparison.validate_registration(self.session(), binding.as_ref(), control)?;
            let outcome =
                self.recompare_fresh_decision_read_set_with(fresh, |evidence, admission| {
                    comparison.compare(execution, self.session(), evidence, admission, control)
                })?;
            comparison.validate_registration(self.session(), binding.as_ref(), control)?;
            Ok(outcome)
        } else {
            self.compare_primary_read_set(execution, fresh.receipt, comparison, control)
        }
    }
}
