use super::super::{
    BridgeOwnedAsyncCompletionAdmission, BridgeOwnedAsyncRequestAdmission, BridgeOwnedSignalRuntime,
};
use crate::facade::{BridgeAsyncCompletionRejection, BridgeAsyncCompletionRejectionKind};
use std::sync::Arc;

/// Genuine committed native completion and its still-owned timeout cleanup.
/// A completion admission report alone cannot construct this result.
pub struct BridgeOwnedAsyncCommittedCompletion {
    authority: Arc<()>,
    native: worth_signal::facade::branch::SignalOwnedAsyncCompletionCommit,
    port: super::super::signal_port::BridgeConditionalSignalPort,
    source: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
}

impl BridgeOwnedAsyncCommittedCompletion {
    pub fn report(&self) -> &worth_signal::facade::ResourceCompletionCommitReport {
        self.native.report()
    }

    pub fn timeout_cleanup_complete(&self) -> bool {
        self.native.timeout_cleanup_complete()
    }

    pub fn timeout_cleanup_denial(
        &self,
    ) -> Option<&worth_signal::facade::branch::SignalConditionalServiceExecutionDenial> {
        self.native.timeout_cleanup_denial()
    }
}

impl BridgeOwnedSignalRuntime {
    pub fn commit_owned_async_completion(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        completion: &BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<BridgeOwnedAsyncCommittedCompletion, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        if !completion.is_owned_by(&self.async_observation_authority) {
            return Err(super::foreign_continuation_denial());
        }
        let admitted = completion
            .report()
            .admitted_completion()
            .filter(|admitted| {
                admitted.completion_class() == crate::facade::BridgeAsyncCompletionClass::Fulfilled
                    && admitted.request_identity() == request.request()
                    && admitted.admitted_completion().handle() == request.request().request_handle()
                    && admitted.admitted_completion().attempt() == request.request().attempt()
            })
            .ok_or_else(|| {
                commit_denial("completion is not this owner's admitted ordinary completion")
            })?;
        let (port, source) = request.signal_parts();
        let native = port
            .commit_owned_async_completion(source, admitted.admitted_completion())
            .map_err(|cause| {
                commit_denial(format!("Signal completion commit refused: {cause:?}"))
            })?;
        Ok(BridgeOwnedAsyncCommittedCompletion {
            authority: Arc::clone(&self.async_observation_authority),
            native,
            port: port.clone(),
            source: source.clone(),
        })
    }

    pub fn finish_owned_async_completion_timeout_cleanup(
        &self,
        committed: &mut BridgeOwnedAsyncCommittedCompletion,
    ) -> Result<(), BridgeAsyncCompletionRejection> {
        if !Arc::ptr_eq(&committed.authority, &self.async_observation_authority) {
            return Err(super::foreign_continuation_denial());
        }
        committed
            .port
            .finish_owned_async_completion_timeout_cleanup(&committed.source, &mut committed.native)
            .map_err(|cause| commit_denial(format!("committed timeout cleanup refused: {cause:?}")))
    }
}

fn commit_denial(detail: impl Into<Arc<str>>) -> BridgeAsyncCompletionRejection {
    BridgeAsyncCompletionRejection::new(
        BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
        detail,
    )
}
