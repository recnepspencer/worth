use std::sync::Arc;

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::{SignalBranchCellIncarnation, SignalOwner};
use crate::branch::AdmittedSignalBranchBasis;
use crate::data::resource::{
    InFlightResourceRequest, RawCompletionEnvelope, ResourceCancellationReason,
    ResourceCancellationReport, ResourceCompletionAdmissionReport, ResourceNodeId,
    ResourceRequestAdmissionReport, ResourceRequestHandle, ResourceRetryAdmissionReport,
    ResourceRetryScheduleReport, ResourceRevalidationIntent, ResourceRevalidationReport,
    ResourceTimeoutReport,
};

use super::issuance::SignalConditionalServiceAuthority;
use super::{
    SignalConditionalExecutionPort, SignalConditionalServiceExecutionDenial,
    SignalInstalledDefinitionBinding,
};

/// Opaque proof that one presealed resource definition belongs to this exact
/// conditional service occurrence. Copying node text cannot manufacture it.
#[derive(Clone)]
pub struct SignalOwnedAsyncSourceBinding {
    node: ResourceNodeId,
    basis: AdmittedSignalBranchBasis,
    authority: Arc<SignalConditionalServiceAuthority>,
    definition: SignalInstalledDefinitionBinding,
    incarnation: SignalBranchCellIncarnation,
}

/// Signal-owned request result retained by Bridge without graph access.
pub struct SignalOwnedAsyncRequestAdmission {
    report: ResourceRequestAdmissionReport,
    in_flight: InFlightResourceRequest,
}

pub struct SignalOwnedAsyncRevalidationAdmission {
    report: ResourceRevalidationReport,
    in_flight: Option<InFlightResourceRequest>,
}

pub struct SignalOwnedAsyncTimeoutAdmission {
    report: ResourceTimeoutReport,
}

pub struct SignalOwnedAsyncRetrySchedule {
    report: ResourceRetryScheduleReport,
}

pub struct SignalOwnedAsyncRetryAdmission {
    report: ResourceRetryAdmissionReport,
    in_flight: Option<InFlightResourceRequest>,
}

impl SignalOwnedAsyncRequestAdmission {
    pub(crate) fn new(
        report: ResourceRequestAdmissionReport,
        in_flight: InFlightResourceRequest,
    ) -> Self {
        Self { report, in_flight }
    }

    pub fn report(&self) -> &ResourceRequestAdmissionReport {
        &self.report
    }

    pub fn in_flight(&self) -> &InFlightResourceRequest {
        &self.in_flight
    }
}

impl SignalOwnedAsyncRevalidationAdmission {
    pub(crate) fn new(
        report: ResourceRevalidationReport,
        in_flight: Option<InFlightResourceRequest>,
    ) -> Self {
        Self { report, in_flight }
    }

    pub fn report(&self) -> &ResourceRevalidationReport {
        &self.report
    }

    pub fn in_flight(&self) -> Option<&InFlightResourceRequest> {
        self.in_flight.as_ref()
    }
}

impl SignalOwnedAsyncTimeoutAdmission {
    pub(crate) fn new(report: ResourceTimeoutReport) -> Self {
        Self { report }
    }

    pub fn report(&self) -> &ResourceTimeoutReport {
        &self.report
    }
}

impl SignalOwnedAsyncRetrySchedule {
    pub(crate) fn new(report: ResourceRetryScheduleReport) -> Self {
        Self { report }
    }

    pub fn report(&self) -> &ResourceRetryScheduleReport {
        &self.report
    }
}

impl SignalOwnedAsyncRetryAdmission {
    pub(crate) fn new(
        report: ResourceRetryAdmissionReport,
        in_flight: Option<InFlightResourceRequest>,
    ) -> Self {
        Self { report, in_flight }
    }

    pub fn report(&self) -> &ResourceRetryAdmissionReport {
        &self.report
    }

    pub fn in_flight(&self) -> Option<&InFlightResourceRequest> {
        self.in_flight.as_ref()
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn bind_owned_async_source(
        &self,
        basis: &AdmittedSignalBranchBasis,
        node: ResourceNodeId,
    ) -> Result<SignalOwnedAsyncSourceBinding, SignalConditionalServiceExecutionDenial> {
        self.with_owned_async_cell(basis, |cell, admission| {
            cell.admit_conditional_owned_async_source(admission, basis, &self.definition, node)
        })?;
        Ok(SignalOwnedAsyncSourceBinding {
            node,
            basis: basis.clone(),
            authority: Arc::clone(&self.authority),
            definition: self.definition.clone(),
            incarnation: self.incarnation,
        })
    }

    pub fn admit_owned_async_request(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
    ) -> Result<SignalOwnedAsyncRequestAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.admit_conditional_owned_async_request(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
            )
        })
    }

    pub fn admit_owned_async_completion(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        raw: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionAdmissionReport, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.admit_conditional_owned_async_completion(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                raw,
            )
        })
    }

    pub fn revalidate_owned_async_request(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncRevalidationAdmission, SignalConditionalServiceExecutionDenial>
    {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.revalidate_conditional_owned_async_request(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                intent,
            )
        })
    }

    pub fn advance_owned_async_request_to_timeout(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        handle: ResourceRequestHandle,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncTimeoutAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.advance_conditional_owned_async_request_to_timeout(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                handle,
                coordinate,
            )
        })
    }

    pub fn schedule_owned_async_retry(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        handle: ResourceRequestHandle,
    ) -> Result<SignalOwnedAsyncRetrySchedule, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.schedule_conditional_owned_async_retry(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                handle,
            )
        })
    }

    pub fn advance_owned_async_retry(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        handle: ResourceRequestHandle,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncRetryAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.advance_conditional_owned_async_retry(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                handle,
                schedule,
                coordinate,
            )
        })
    }

    pub fn active_owned_async_request_count(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
    ) -> Result<usize, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.conditional_owned_async_active_request_count(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
            )
        })
    }

    pub fn cancel_owned_async_request(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        handle: crate::data::resource::ResourceRequestHandle,
    ) -> Result<ResourceCancellationReport, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.cancel_conditional_owned_async_request(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                handle,
                ResourceCancellationReason::HostRequested,
            )
        })
    }

    fn require_owned_async_binding(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        if !Arc::ptr_eq(&self.authority, &binding.authority)
            || self.incarnation != binding.incarnation
            || !self.definition.matches(&binding.definition)
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        Ok(())
    }

    fn with_owned_async_cell<Output>(
        &self,
        basis: &AdmittedSignalBranchBasis,
        operation: impl FnOnce(
            &crate::branch::owner_services::SignalBranchExecutionCell<
                crate::branch::owner_services::SignalBranchCellState<D, I, T>,
            >,
            &crate::branch::owner_services::SignalOwnerOperationAdmission<'_>,
        ) -> Result<Output, SignalConditionalServiceExecutionDenial>,
    ) -> Result<Output, SignalConditionalServiceExecutionDenial> {
        use SignalConditionalServiceExecutionDenial as Denial;
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        if cell.incarnation() != self.incarnation {
            return Err(Denial::StaleBasisAdmission);
        }
        operation(&cell, &admission)
    }
}
