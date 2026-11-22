//! Fresh session admission of the primary owner's completed original prefix.
use std::sync::Arc;
use worth_query_execution_runtime::facade::ports::{
    DecisionReadSetReceiptInternalPort as _, FreshDecisionReadSetInternalPort as _,
    PrimaryReadCustody,
};
use worth_query_execution_runtime::facade::provider_session::{
    WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetDenialKind,
    WorthQueryDecisionReadSetFailure, WorthQueryDecisionReadSetFreshnessOutcome,
    WorthQueryFreshDecisionReadSet, WorthQuerySessionReadAuthority,
};
use worth_query_installation::facade::{
    WorthQueryDecisionFactCardinality, WorthQueryDecisionFactKind, APPLICATION_DECISION_FACT_FAMILY,
};

use crate::domain_computation::primary_graph::{
    PrimaryDecisionReadSetComparison, RegisteredPrimaryRead, StorageControl,
};

fn denial(kind: WorthQueryDecisionReadSetDenialKind) -> WorthQueryDecisionReadSetFailure {
    WorthQueryDecisionReadSetFailure::new(kind, "decision read-set authority denied")
}

fn check_request(
    request: Option<
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    >,
) -> Result<(), WorthQueryDecisionReadSetFailure> {
    match request.and_then(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::interruption,
    ) {
        Some(stop) => Err(denial(
            WorthQueryDecisionReadSetDenialKind::RequestInterrupted(stop),
        )),
        None => Ok(()),
    }
}

fn downcast_read(custody: &Arc<dyn PrimaryReadCustody>) -> RegisteredPrimaryRead {
    custody
        .as_any()
        .downcast_ref::<RegisteredPrimaryRead>()
        .expect("primary read custody belongs to RegisteredPrimaryRead")
        .clone()
}

pub trait PrimaryReadSetCaptureExt {
    fn capture_primary_read_set(
        &self,
        read: RegisteredPrimaryRead,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure>;

    fn compare_primary_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: WorthQueryCompleteDecisionReadSetReceipt,
        comparison: &mut PrimaryDecisionReadSetComparison<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure>;

    fn recompare_primary_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh: WorthQueryFreshDecisionReadSet,
        comparison: &mut PrimaryDecisionReadSetComparison<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure>;
}

fn capture_primary_full(
    authority: &WorthQuerySessionReadAuthority<'_>,
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
    let mut receipt = authority.capture_decision_read_set(
        table.requests(),
        control.policy(),
        control.request(),
    )?;
    receipt.attach_primary_full(Arc::new(read), memory);
    Ok(receipt)
}

impl PrimaryReadSetCaptureExt for WorthQuerySessionReadAuthority<'_> {
    fn capture_primary_read_set(
        &self,
        read: RegisteredPrimaryRead,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryCompleteDecisionReadSetReceipt, WorthQueryDecisionReadSetFailure> {
        let table = read.facts();
        let Some(_) = table.closed_prefix() else {
            return capture_primary_full(self, read, control);
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
            return capture_primary_full(self, read, control);
        }
        let mut receipt = self.capture_decision_read_set(
            table.explicit_requests(),
            control.policy(),
            control.request(),
        )?;
        let grouped_count = table.grouped_count();
        let shared: Arc<dyn PrimaryReadCustody> = Arc::new(read);
        receipt
            .attach_primary_prefix(grouped_count, shared)
            .map_err(|()| denial(WorthQueryDecisionReadSetDenialKind::FactCountOverflow))?;
        Ok(receipt)
    }

    fn compare_primary_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut receipt: WorthQueryCompleteDecisionReadSetReceipt,
        comparison: &mut PrimaryDecisionReadSetComparison<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        self.validate_decision_read_set_receipt(&receipt)?;
        let binding = receipt.primary_custody().map(downcast_read);
        comparison.validate_registration(self.session(), binding.as_ref(), control)?;
        let primary = receipt.take_primary();
        let primary_read = primary.as_ref().map(downcast_read);
        if let Some(table) = &primary_read
            && !comparison.closes_original(execution, self.session(), table, control)?
        {
            let full = capture_primary_full(self, table.clone(), control)?;
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
            if let Some(table) = &primary_read
                && !comparison.closes_original(execution, self.session(), table, control)?
            {
                return Err(denial(
                    WorthQueryDecisionReadSetDenialKind::EvidenceSubstitution,
                ));
            }
            fresh.set_primary(primary);
        }
        comparison.validate_registration(self.session(), binding.as_ref(), control)?;
        Ok(outcome)
    }

    fn recompare_primary_read_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fresh: WorthQueryFreshDecisionReadSet,
        comparison: &mut PrimaryDecisionReadSetComparison<'_>,
        control: StorageControl<'_, '_>,
    ) -> Result<WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryDecisionReadSetFailure> {
        if !fresh.has_primary() {
            let binding = fresh.primary_custody().map(downcast_read);
            comparison.validate_registration(self.session(), binding.as_ref(), control)?;
            let outcome =
                self.recompare_fresh_decision_read_set_with(fresh, |evidence, admission| {
                    comparison.compare(execution, self.session(), evidence, admission, control)
                })?;
            comparison.validate_registration(self.session(), binding.as_ref(), control)?;
            Ok(outcome)
        } else {
            self.compare_primary_read_set(execution, fresh.into_receipt(), comparison, control)
        }
    }
}
