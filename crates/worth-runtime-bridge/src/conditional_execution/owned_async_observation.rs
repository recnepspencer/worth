use std::sync::Arc;

use crate::facade::AdmittedBridgeAsyncRequestIdentity;

pub struct BridgeOwnedAsyncRequestAdmission {
    authority: Arc<()>,
    request: AdmittedBridgeAsyncRequestIdentity,
    signal_port: super::signal_port::BridgeConditionalSignalPort,
    signal_binding: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
    effects_indeterminate: BridgeOwnedAsyncEffectsIndeterminateIssuer,
}

pub struct BridgeOwnedAsyncSupersessionAdmission<'a> {
    prior: &'a BridgeOwnedAsyncRequestAdmission,
    displacing: &'a BridgeOwnedAsyncRequestAdmission,
}

#[derive(Debug)]
pub struct BridgeOwnedAsyncCompletionAdmission {
    authority: Arc<()>,
    report: crate::facade::BridgeAsyncCompletionAdmissionReport,
}

pub struct BridgeOwnedAsyncTimeoutAdmission {
    authority: Arc<()>,
    request_handle: worth_signal::facade::ResourceRequestHandle,
    report: worth_signal::facade::ResourceTimeoutReport,
}

pub struct BridgeOwnedAsyncRetrySchedule {
    authority: Arc<()>,
    request_handle: worth_signal::facade::ResourceRequestHandle,
    timeout: worth_signal::facade::ResourceTimeoutReport,
    signal: worth_signal::facade::branch::SignalOwnedAsyncRetrySchedule,
}

pub struct BridgeOwnedAsyncRetryAdmission {
    request: BridgeOwnedAsyncRequestAdmission,
    lineage: crate::facade::BridgeAsyncRetryLineage,
}

pub struct BridgeOwnedAsyncRevalidationAdmission {
    request: BridgeOwnedAsyncRequestAdmission,
    lineage: crate::facade::BridgeAsyncRevalidationLineage,
}

#[derive(Clone)]
pub struct BridgeOwnedAsyncEffectsIndeterminateIssuer {
    authority: Arc<()>,
    request: AdmittedBridgeAsyncRequestIdentity,
    signal_port: super::signal_port::BridgeConditionalSignalPort,
    signal_binding: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
}

pub struct BridgeAsyncEffectsIndeterminateObservation {
    authority: Arc<()>,
    request: AdmittedBridgeAsyncRequestIdentity,
    signal_port: super::signal_port::BridgeConditionalSignalPort,
    signal_binding: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
    envelope: worth_signal::facade::RawCompletionEnvelope,
}

impl BridgeOwnedAsyncRequestAdmission {
    pub(super) fn new(
        authority: &Arc<()>,
        request: AdmittedBridgeAsyncRequestIdentity,
        signal_port: super::signal_port::BridgeConditionalSignalPort,
        signal_binding: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
    ) -> Self {
        Self {
            authority: Arc::clone(authority),
            effects_indeterminate: BridgeOwnedAsyncEffectsIndeterminateIssuer {
                authority: Arc::clone(authority),
                request: request.clone(),
                signal_port: signal_port.clone(),
                signal_binding: signal_binding.clone(),
            },
            request,
            signal_port,
            signal_binding,
        }
    }

    pub fn request(&self) -> &AdmittedBridgeAsyncRequestIdentity {
        &self.request
    }

    pub(super) fn signal_parts(
        &self,
    ) -> (
        &super::signal_port::BridgeConditionalSignalPort,
        &worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
    ) {
        (&self.signal_port, &self.signal_binding)
    }

    pub(super) fn is_owned_by(&self, authority: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.authority, authority)
    }

    pub fn effects_indeterminate_issuer(&self) -> BridgeOwnedAsyncEffectsIndeterminateIssuer {
        self.effects_indeterminate.clone()
    }
}

impl<'a> BridgeOwnedAsyncSupersessionAdmission<'a> {
    pub(super) fn new(
        prior: &'a BridgeOwnedAsyncRequestAdmission,
        displacing: &'a BridgeOwnedAsyncRequestAdmission,
    ) -> Self {
        Self { prior, displacing }
    }

    pub fn prior(&self) -> &'a AdmittedBridgeAsyncRequestIdentity {
        self.prior.request()
    }

    pub fn displacing(&self) -> &'a AdmittedBridgeAsyncRequestIdentity {
        self.displacing.request()
    }
}

impl BridgeOwnedAsyncCompletionAdmission {
    pub(super) fn new(
        authority: &Arc<()>,
        report: crate::facade::BridgeAsyncCompletionAdmissionReport,
    ) -> Self {
        Self {
            authority: Arc::clone(authority),
            report,
        }
    }

    pub fn report(&self) -> &crate::facade::BridgeAsyncCompletionAdmissionReport {
        &self.report
    }

    pub(super) fn is_owned_by(&self, authority: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.authority, authority)
    }
}

impl BridgeOwnedAsyncTimeoutAdmission {
    pub(super) fn new(
        authority: &Arc<()>,
        request_handle: worth_signal::facade::ResourceRequestHandle,
        report: worth_signal::facade::ResourceTimeoutReport,
    ) -> Self {
        Self {
            authority: Arc::clone(authority),
            request_handle,
            report,
        }
    }

    pub fn report(&self) -> &worth_signal::facade::ResourceTimeoutReport {
        &self.report
    }

    pub(super) fn matches(
        &self,
        authority: &Arc<()>,
        request_handle: worth_signal::facade::ResourceRequestHandle,
    ) -> bool {
        Arc::ptr_eq(&self.authority, authority) && self.request_handle == request_handle
    }
}

impl BridgeOwnedAsyncRetrySchedule {
    pub(super) fn new(
        authority: &Arc<()>,
        request_handle: worth_signal::facade::ResourceRequestHandle,
        timeout: worth_signal::facade::ResourceTimeoutReport,
        signal: worth_signal::facade::branch::SignalOwnedAsyncRetrySchedule,
    ) -> Self {
        Self {
            authority: Arc::clone(authority),
            request_handle,
            timeout,
            signal,
        }
    }

    pub fn report(&self) -> &worth_signal::facade::ResourceRetryScheduleReport {
        self.signal.report()
    }

    pub(super) fn matches(
        &self,
        authority: &Arc<()>,
        request_handle: worth_signal::facade::ResourceRequestHandle,
    ) -> bool {
        Arc::ptr_eq(&self.authority, authority) && self.request_handle == request_handle
    }

    pub(super) fn signal(&self) -> &worth_signal::facade::branch::SignalOwnedAsyncRetrySchedule {
        &self.signal
    }

    pub(super) fn timeout(&self) -> &worth_signal::facade::ResourceTimeoutReport {
        &self.timeout
    }
}

impl BridgeOwnedAsyncRetryAdmission {
    pub(super) fn new(
        request: BridgeOwnedAsyncRequestAdmission,
        lineage: crate::facade::BridgeAsyncRetryLineage,
    ) -> Self {
        Self { request, lineage }
    }

    pub fn request(&self) -> &BridgeOwnedAsyncRequestAdmission {
        &self.request
    }

    pub fn lineage(&self) -> &crate::facade::BridgeAsyncRetryLineage {
        &self.lineage
    }

    pub fn into_request(self) -> BridgeOwnedAsyncRequestAdmission {
        self.request
    }
}

impl BridgeOwnedAsyncRevalidationAdmission {
    pub(super) fn new(
        request: BridgeOwnedAsyncRequestAdmission,
        lineage: crate::facade::BridgeAsyncRevalidationLineage,
    ) -> Self {
        Self { request, lineage }
    }

    pub fn request(&self) -> &BridgeOwnedAsyncRequestAdmission {
        &self.request
    }

    pub fn lineage(&self) -> &crate::facade::BridgeAsyncRevalidationLineage {
        &self.lineage
    }

    pub fn into_request(self) -> BridgeOwnedAsyncRequestAdmission {
        self.request
    }
}

impl BridgeOwnedAsyncEffectsIndeterminateIssuer {
    pub fn certify(&self, payload_byte_len: u64) -> BridgeAsyncEffectsIndeterminateObservation {
        let descriptor = self
            .request
            .lowered()
            .resource_descriptor()
            .expect("owner-issued async request retains its resource descriptor");
        BridgeAsyncEffectsIndeterminateObservation {
            authority: Arc::clone(&self.authority),
            request: self.request.clone(),
            signal_port: self.signal_port.clone(),
            signal_binding: self.signal_binding.clone(),
            envelope: worth_signal::facade::RawCompletionEnvelope::new(
                self.request.request_handle().request_id(),
                self.request.request_handle().generation(),
                self.request.request_handle().branch_epoch(),
                self.request.attempt(),
                descriptor.payload_contract_digest().clone(),
                payload_byte_len,
            ),
        }
    }
}

impl BridgeAsyncEffectsIndeterminateObservation {
    pub(super) fn into_parts(
        self,
    ) -> (
        Arc<()>,
        AdmittedBridgeAsyncRequestIdentity,
        super::signal_port::BridgeConditionalSignalPort,
        worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
        worth_signal::facade::RawCompletionEnvelope,
    ) {
        (
            self.authority,
            self.request,
            self.signal_port,
            self.signal_binding,
            self.envelope,
        )
    }
}
