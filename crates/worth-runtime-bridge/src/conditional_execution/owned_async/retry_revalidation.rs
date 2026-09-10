use crate::facade::{
    BridgeAsyncCompletionRejection, BridgeAsyncCompletionRejectionKind,
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestTruthViewBasis,
};

use super::super::{
    BridgeOwnedAsyncRequestAdmission, BridgeOwnedAsyncRetryAdmission,
    BridgeOwnedAsyncRetrySchedule, BridgeOwnedAsyncRevalidationAdmission,
    BridgeOwnedAsyncTimeoutAdmission, BridgeOwnedSignalRuntime,
};
use super::foreign_continuation_denial;

impl BridgeOwnedSignalRuntime {
    pub fn advance_owned_async_request_to_timeout(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        coordinate: u64,
    ) -> Result<BridgeOwnedAsyncTimeoutAdmission, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        let (port, binding) = request.signal_parts();
        let signal = port
            .advance_owned_async_request_to_timeout(
                binding,
                request.request().request_handle(),
                coordinate,
            )
            .map_err(signal_lifecycle_denial)?;
        if signal.report().timed_out_request().is_none() {
            return Err(signal_lifecycle_denial(
                "Signal denied the owned async timeout admission",
            ));
        }
        Ok(BridgeOwnedAsyncTimeoutAdmission::new(
            &self.async_observation_authority,
            request.request().request_handle(),
            signal.report().clone(),
        ))
    }

    pub fn schedule_owned_async_retry(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        timeout: &BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<BridgeOwnedAsyncRetrySchedule, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        if !timeout.matches(
            &self.async_observation_authority,
            request.request().request_handle(),
        ) {
            return Err(foreign_continuation_denial());
        }
        let (port, binding) = request.signal_parts();
        let signal = port
            .schedule_owned_async_retry(binding, request.request().request_handle())
            .map_err(signal_lifecycle_denial)?;
        if signal.report().scheduled_retry().is_none() {
            return Err(signal_lifecycle_denial(format!(
                "Signal denied the configured owned async retry: {:?}",
                signal.report().denied_retry()
            )));
        }
        Ok(BridgeOwnedAsyncRetrySchedule::new(
            &self.async_observation_authority,
            request.request().request_handle(),
            timeout.report().clone(),
            signal,
        ))
    }

    pub fn advance_owned_async_retry(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        schedule: &BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<BridgeOwnedAsyncRetryAdmission, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        if !schedule.matches(
            &self.async_observation_authority,
            request.request().request_handle(),
        ) {
            return Err(foreign_continuation_denial());
        }
        let (port, binding) = request.signal_parts();
        let signal = port
            .advance_owned_async_retry(
                binding,
                request.request().request_handle(),
                schedule.signal(),
                coordinate,
            )
            .map_err(signal_lifecycle_denial)?;
        let admitted = signal.report().admitted_retry().ok_or_else(|| {
            signal_lifecycle_denial(format!(
                "Signal denied the scheduled owned async retry: {:?}",
                signal.report().denied_retry()
            ))
        })?;
        let in_flight = signal.in_flight().cloned().ok_or_else(|| {
            signal_lifecycle_denial("Signal retry admission lost its in-flight request")
        })?;
        let rebind = BridgeAsyncRequestAdmissionRequest::rebind(
            request.request().lowered(),
            request.request().basis_binding(),
            request.request().family_admission(),
        )
        .map_err(request_lifecycle_denial)?;
        let newer = crate::source::admit_from_owned_signal_request(
            self.bridge.signal_runtime_key,
            rebind,
            admitted.admitted_request(),
            in_flight,
        )
        .map_err(request_lifecycle_denial)?;
        let lineage = crate::source::admit_owned_retry_lineage(
            request.request().clone(),
            newer.clone(),
            schedule.timeout(),
            schedule.report(),
            signal.report(),
        )
        .map_err(forward_lifecycle_denial)?;
        Ok(BridgeOwnedAsyncRetryAdmission::new(
            BridgeOwnedAsyncRequestAdmission::new(
                &self.async_observation_authority,
                newer,
                port.clone(),
                binding.clone(),
            ),
            lineage,
        ))
    }

    pub fn revalidate_owned_async_request(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<BridgeOwnedAsyncRevalidationAdmission, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        if request
            .request()
            .basis_binding()
            .truth_view_basis()
            .digest()
            == truth_basis.digest()
        {
            return Err(signal_lifecycle_denial(
                "owned async revalidation requires a changed product truth basis",
            ));
        }
        let bound = self
            .bind_owned_async_basis(request.request().lowered(), signal_basis, truth_basis)
            .map_err(request_lifecycle_denial)?;
        let node = request
            .request()
            .lowered()
            .resource_descriptor()
            .expect("owned request-response lowering retains its descriptor")
            .node();
        let signal = bound
            .port
            .revalidate_owned_async_request(
                &bound.source,
                worth_signal::facade::ResourceRevalidationIntent::with_expected_active(
                    node,
                    request.request().request_handle(),
                ),
            )
            .map_err(signal_lifecycle_denial)?;
        let admitted = signal.report().admitted_revalidation().ok_or_else(|| {
            signal_lifecycle_denial(format!(
                "Signal denied owned async revalidation: {:?}",
                signal.report().denied_revalidation()
            ))
        })?;
        let in_flight = signal.in_flight().cloned().ok_or_else(|| {
            signal_lifecycle_denial("Signal revalidation lost its in-flight request")
        })?;
        let newer = crate::source::admit_from_owned_signal_request(
            self.bridge.signal_runtime_key,
            bound.request,
            admitted.admitted_request(),
            in_flight,
        )
        .map_err(request_lifecycle_denial)?;
        let lineage = crate::source::admit_owned_revalidation_lineage(
            request.request().clone(),
            newer.clone(),
            signal.report(),
        )
        .map_err(forward_lifecycle_denial)?;
        Ok(BridgeOwnedAsyncRevalidationAdmission::new(
            BridgeOwnedAsyncRequestAdmission::new(
                &self.async_observation_authority,
                newer,
                bound.port,
                bound.source,
            ),
            lineage,
        ))
    }
}

fn signal_lifecycle_denial(detail: impl std::fmt::Debug) -> BridgeAsyncCompletionRejection {
    BridgeAsyncCompletionRejection::new(
        BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
        format!("owned async lifecycle denied: {detail:?}"),
    )
}

fn request_lifecycle_denial(
    denial: crate::facade::BridgeAsyncRequestIdentityRejection,
) -> BridgeAsyncCompletionRejection {
    signal_lifecycle_denial(denial)
}

fn forward_lifecycle_denial(
    denial: crate::facade::BridgeAsyncForwardCausalityRejection,
) -> BridgeAsyncCompletionRejection {
    signal_lifecycle_denial(denial)
}
