use std::sync::Arc;

use crate::facade::{
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestIdentityRejection,
    BridgeAsyncRequestIdentityRejectionKind, BridgeAsyncRequestTruthViewBasis,
    LoweredBridgeAsyncSourceDeclaration,
};

use super::super::BridgeOwnedSignalRuntime;

pub(super) struct BridgeBoundOwnedAsyncBasis {
    pub(super) request: BridgeAsyncRequestAdmissionRequest,
    pub(super) port: super::super::signal_port::BridgeConditionalSignalPort,
    pub(super) source: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
}

impl BridgeOwnedSignalRuntime {
    pub fn admit_owned_async_request_identity(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<super::super::BridgeOwnedAsyncRequestAdmission, BridgeAsyncRequestIdentityRejection>
    {
        let bound = self.bind_owned_async_basis(lowered, signal_basis, truth_basis)?;
        let admitted = bound
            .port
            .admit_owned_async_request(&bound.source)
            .map_err(|denial| request_denial(format!("Signal request denied: {denial:?}")))?;
        let request = crate::source::admit_from_owned_signal_request(
            self.bridge.signal_runtime_key,
            bound.request,
            admitted.report().admitted_request(),
            admitted.in_flight().clone(),
        )?;
        Ok(super::super::BridgeOwnedAsyncRequestAdmission::new(
            &self.async_observation_authority,
            request,
            bound.port,
            bound.source,
        ))
    }

    pub(super) fn bind_owned_async_basis(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<BridgeBoundOwnedAsyncBasis, BridgeAsyncRequestIdentityRejection> {
        let installed = self
            .installed_owned_async_declaration(lowered.declaration_identity().as_str())
            .filter(|installed| installed == lowered)
            .ok_or_else(|| request_denial("owned async declaration is not installed here"))?;
        let binding = self
            .bridge
            .bind_async_request_basis(&installed, truth_basis);
        let request = BridgeAsyncRequestAdmissionRequest::request_response(&installed, &binding)?;
        let descriptor = installed.resource_descriptor().ok_or_else(|| {
            request_denial("installed owned async resource descriptor is missing")
        })?;
        let installed_port = self
            .signal_services()
            .map_err(|denial| request_denial(denial.detail()))?
            .conditional_extension_port();
        let port = if installed_port.issuance_basis().admission_identity()
            == signal_basis.admission_identity()
        {
            installed_port
        } else {
            super::super::signal_port::BridgeConditionalSignalPort::shared(Arc::new(
                installed_port
                    .reissue_for_successor_basis(signal_basis)
                    .map_err(|denial| {
                        request_denial(format!(
                            "Signal selected-product service admission denied: {denial:?}"
                        ))
                    })?,
            ))
        };
        let source = port
            .bind_owned_async_source(signal_basis, descriptor.node())
            .map_err(|denial| {
                request_denial(format!("Signal source binding denied: {denial:?}"))
            })?;
        Ok(BridgeBoundOwnedAsyncBasis {
            request,
            port,
            source,
        })
    }
}

fn request_denial(detail: impl Into<Arc<str>>) -> BridgeAsyncRequestIdentityRejection {
    BridgeAsyncRequestIdentityRejection::new(
        BridgeAsyncRequestIdentityRejectionKind::SignalRequestAdmissionRejected,
        detail,
    )
}
