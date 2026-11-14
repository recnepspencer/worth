use std::sync::Arc;

use crate::facade::{
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestIdentityRejection,
    BridgeAsyncRequestIdentityRejectionKind, BridgeAsyncRequestTruthViewBasis,
    LoweredBridgeAsyncSourceDeclaration,
};

use super::super::BridgeConditionalExecutionView;

pub(super) struct BridgeBoundOwnedAsyncBasis {
    pub(super) request: BridgeAsyncRequestAdmissionRequest,
    pub(super) port: super::super::signal_port::BridgeConditionalSignalPort,
    pub(super) source: worth_signal::facade::branch::SignalOwnedAsyncSourceBinding,
}

impl BridgeConditionalExecutionView<'_> {
    fn owned_async_port(
        &self,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<
        super::super::signal_port::BridgeConditionalSignalPort,
        BridgeAsyncRequestIdentityRejection,
    > {
        if let Some(owner) = self.durable_owner {
            let source = self
                .source_owner()
                .map_err(|cause| request_denial(cause.detail()))?;
            let port = owner
                .issue_conditional_execution_service(
                    basis,
                    &self.bridge.signal_aspect_lowering_owner,
                    &source.authority(),
                )
                .map_err(|cause| {
                    request_denial(format!(
                        "authored owned-async service issuance denied: {cause:?}"
                    ))
                })?;
            return Ok(
                super::super::signal_port::BridgeConditionalSignalPort::shared(Arc::new(port)),
            );
        }
        let installed = self
            .signal_services
            .ok_or_else(|| {
                request_denial("owned async receiver has no sealed Signal source services")
            })?
            .conditional_extension_port();
        if installed.issuance_basis().admission_identity() == basis.admission_identity() {
            Ok(installed)
        } else {
            let port = installed
                .reissue_for_successor_basis(basis)
                .map_err(|cause| {
                    request_denial(format!(
                        "Signal selected-product service admission denied: {cause:?}"
                    ))
                })?;
            Ok(super::super::signal_port::BridgeConditionalSignalPort::shared(Arc::new(port)))
        }
    }

    fn resolve_owned_async_definition(
        &self,
        expected: &LoweredBridgeAsyncSourceDeclaration,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        port: &super::super::signal_port::BridgeConditionalSignalPort,
    ) -> Result<LoweredBridgeAsyncSourceDeclaration, BridgeAsyncRequestIdentityRejection> {
        if let Some(installed) =
            self.installed_owned_async_declaration(expected.declaration_identity().as_str())
        {
            return if installed.digest() == expected.digest() {
                Ok(installed)
            } else {
                Err(request_denial(
                    "owned async declaration meaning differs from this receiver",
                ))
            };
        }
        let source = self
            .async_definitions
            .and_then(|rows| rows.get(expected.declaration_identity().as_str()))
            .ok_or_else(|| request_denial("owned async declaration is not installed here"))?;
        let declaration = super::BridgeOwnedAsyncRequestResponseDeclaration::from_description(
            source.description.clone(),
        );
        let resource = declaration
            .signal_declaration(source.node)
            .map_err(|cause| request_denial(cause.detail()))?;
        let draft = crate::facade::BridgeAsyncSourceDeclarationDraft::request_response(
            crate::facade::BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(
                declaration.identity.clone(),
            ),
            crate::facade::BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(format!(
                "owned-async-internal:{}",
                declaration.identity
            )),
            resource.clone(),
        );
        let validated = self
            .bridge
            .validate_async_source_declaration(draft)
            .map_err(|cause| request_denial(cause.detail()))?;
        let descriptor = port
            .readmit_owned_async_source_definition(basis, resource.clone())
            .map_err(|cause| {
                request_denial(format!(
                    "Native resource definition readmission refused: {cause:?}"
                ))
            })?;
        let installed = LoweredBridgeAsyncSourceDeclaration::from_request_response_parts(
            &validated, resource, descriptor,
        );
        if installed.digest() != expected.digest() {
            return Err(request_denial(
                "recorded owned async definition differs from the requested meaning",
            ));
        }
        self.async_declarations
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                declaration.identity,
                super::BridgeInstalledOwnedAsyncDeclaration {
                    lowered: installed.clone(),
                    source: source.clone(),
                },
            );
        Ok(installed)
    }

    pub(super) fn bind_owned_async_basis(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<BridgeBoundOwnedAsyncBasis, BridgeAsyncRequestIdentityRejection> {
        let port = self.owned_async_port(signal_basis)?;
        let installed = self.resolve_owned_async_definition(lowered, signal_basis, &port)?;
        let binding = self
            .bridge
            .bind_async_request_basis(&installed, truth_basis);
        let request = BridgeAsyncRequestAdmissionRequest::request_response(&installed, &binding)?;
        let descriptor = installed.resource_descriptor().ok_or_else(|| {
            request_denial("installed owned async resource descriptor is missing")
        })?;
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

pub(super) fn request_denial(detail: impl Into<Arc<str>>) -> BridgeAsyncRequestIdentityRejection {
    BridgeAsyncRequestIdentityRejection::new(
        BridgeAsyncRequestIdentityRejectionKind::SignalRequestAdmissionRejected,
        detail,
    )
}
