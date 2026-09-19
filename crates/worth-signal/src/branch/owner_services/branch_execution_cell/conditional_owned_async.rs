use std::sync::atomic::Ordering;

use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceExecutionDenial, SignalInstalledDefinitionBinding,
    SignalOwnedAsyncRequestAdmission, SignalOwnedAsyncRetryAdmission,
    SignalOwnedAsyncRetrySchedule, SignalOwnedAsyncTimeoutAdmission,
};
use crate::branch::owner_services::{SignalBranchCellState, SignalOwnerOperationAdmission};
use crate::branch::AdmittedSignalBranchBasis;
use crate::data::resource::{
    RawCompletionEnvelope, ResourceCancellationReason, ResourceCancellationReport,
    ResourceCompletionAdmissionReport, ResourceNodeId, ResourceRequestHandle,
    ResourceRevalidationIntent,
};

use super::basis::map_basis_cell_denial;
use super::{SignalBranchCellAdmissionDenial, SignalBranchExecutionCell};

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn admit_conditional_owned_async_source(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.admit_owned_async_source(node)
        })
    }

    pub(in crate::branch::owner_services) fn admit_conditional_owned_async_request(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
    ) -> Result<SignalOwnedAsyncRequestAdmission, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.admit_owned_async_request(node)
        })
    }

    pub(in crate::branch::owner_services) fn admit_conditional_owned_async_completion(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        raw: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionAdmissionReport, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.admit_owned_async_completion(node, raw)
        })
    }

    pub(in crate::branch::owner_services) fn revalidate_conditional_owned_async_request(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        intent: ResourceRevalidationIntent,
    ) -> Result<
        crate::branch::owner_services::conditional_execution::SignalOwnedAsyncRevalidationAdmission,
        SignalConditionalServiceExecutionDenial,
    > {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.revalidate_owned_async_request(node, intent)
        })
    }

    pub(in crate::branch::owner_services) fn advance_conditional_owned_async_request_to_timeout(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncTimeoutAdmission, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.advance_owned_async_request_to_timeout(node, handle, coordinate)
        })
    }

    pub(in crate::branch::owner_services) fn schedule_conditional_owned_async_retry(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<SignalOwnedAsyncRetrySchedule, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.schedule_owned_async_retry(node, handle)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::branch::owner_services) fn advance_conditional_owned_async_retry(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncRetryAdmission, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.advance_owned_async_retry(node, handle, schedule, coordinate)
        })
    }

    pub(in crate::branch::owner_services) fn conditional_owned_async_active_request_count(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
    ) -> Result<usize, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.owned_async_active_request_count(node)
        })
    }

    pub(in crate::branch::owner_services) fn cancel_conditional_owned_async_request(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        reason: ResourceCancellationReason,
    ) -> Result<ResourceCancellationReport, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.cancel_owned_async_request(node, handle, reason)
        })
    }

    fn with_conditional_owned_async_state<Output>(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        operation: impl FnOnce(
            &mut SignalBranchCellState<D, I, T>,
        ) -> Result<Output, SignalConditionalServiceExecutionDenial>,
    ) -> Result<Output, SignalConditionalServiceExecutionDenial> {
        use SignalConditionalServiceExecutionDenial as Denial;
        let map_cell =
            |denial| Denial::OwnerAdmission(map_basis_cell_denial(denial, self.branch_id));
        self.validate_admission(admission).map_err(map_cell)?;
        let _hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(map_cell)?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        self.require_live_posture().map_err(map_cell)?;
        let mut state = self
            .lock_state_after_contention_observation()
            .map_err(map_cell)?;
        self.require_live_posture().map_err(map_cell)?;
        state
            .observation()
            .map_err(|error| {
                Denial::OwnerAdmission(
                    crate::branch::SignalBranchBasisObservationDenial::InvalidOwnerObservation {
                        error,
                    },
                )
            })?
            .compare(basis.observation())
            .map_err(|_| Denial::StaleBasisAdmission)?;
        let current = state
            .state()
            .installed_definition()
            .ok_or(Denial::DefinitionReadmissionRequired)?;
        if !current.matches(definition) {
            return Err(Denial::DefinitionMismatch);
        }
        operation(&mut state)
    }
}
