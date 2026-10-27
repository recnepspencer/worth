use super::super::{
    BridgeOwnedAsyncRetrySchedule, BridgeOwnedAsyncTimeoutAdmission, BridgeOwnedSignalRuntime,
};
use super::{
    BridgeOwnedAsyncManagedDenial as Denial, BridgeOwnedAsyncManagedRequestAdmission as Custodian,
};
use worth_signal::facade::branch::{
    SignalOwnedAsyncManagedProgress, SignalOwnedAsyncManagedReport,
};

impl BridgeOwnedSignalRuntime {
    pub fn advance_managed_owned_async_request_to_timeout(
        &self,
        owner: &mut Custodian,
        coordinate: u64,
    ) -> Result<BridgeOwnedAsyncTimeoutAdmission, Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        let port = owner.port.clone();
        port.advance_managed_owned_async_request_to_timeout(&mut owner.native, coordinate)
            .map_err(Denial::Signal)?;
        self.retain_managed_native_progress(owner, port, None)?;
        let SignalOwnedAsyncManagedReport::Timeout(report) = owner.native.report() else {
            return Err(Denial::ProgressDenied);
        };
        if report.timed_out_request().is_none() {
            return Err(Denial::ProgressDenied);
        }
        Ok(BridgeOwnedAsyncTimeoutAdmission::new(
            &self.async_observation_authority,
            owner.request_handle(),
            report.clone(),
        ))
    }

    pub fn schedule_managed_owned_async_retry(
        &self,
        owner: &mut Custodian,
        timeout: &BridgeOwnedAsyncTimeoutAdmission,
    ) -> Result<BridgeOwnedAsyncRetrySchedule, Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        if !timeout.matches(&self.async_observation_authority, owner.request_handle()) {
            return Err(Denial::ForeignOwner);
        }
        let port = owner.port.clone();
        let native = port
            .schedule_managed_owned_async_retry(&mut owner.native)
            .map_err(Denial::Signal)?;
        self.retain_managed_native_progress(owner, port, None)?;
        if native.report().scheduled_retry().is_none() {
            return Err(Denial::ProgressDenied);
        }
        Ok(BridgeOwnedAsyncRetrySchedule::new(
            &self.async_observation_authority,
            owner.request_handle(),
            timeout.report().clone(),
            native,
        ))
    }

    pub fn advance_managed_owned_async_retry(
        &self,
        owner: &mut Custodian,
        schedule: &BridgeOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<crate::facade::BridgeAsyncRetryLineage, Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        if !schedule.matches(&self.async_observation_authority, owner.request_handle()) {
            return Err(Denial::ForeignOwner);
        }
        let prior = owner
            .request
            .as_ref()
            .map(|request| request.request().clone());
        let port = owner.port.clone();
        let source = owner.native.source_binding().clone();
        let progress = port
            .advance_managed_owned_async_retry(
                &source,
                &mut owner.native,
                schedule.signal(),
                coordinate,
            )
            .map_err(Denial::Signal)?;
        self.retain_managed_native_progress(owner, port, None)?;
        if progress != SignalOwnedAsyncManagedProgress::Replaced {
            return Err(Denial::ProgressDenied);
        }
        let prior = prior.ok_or(Denial::RequestUnavailable)?;
        let current = owner.request.as_ref().ok_or(Denial::RequestUnavailable)?;
        let SignalOwnedAsyncManagedReport::Retry(report) = owner.native.report() else {
            unreachable!("native retry replacement retains its report")
        };
        crate::source::admit_owned_retry_lineage(
            prior,
            current.request().clone(),
            schedule.timeout(),
            schedule.report(),
            report,
        )
        .map_err(|cause| {
            owner.refusal = Some(Denial::Lineage(cause.clone()));
            Denial::Lineage(cause)
        })
    }
}
