use std::sync::PoisonError;

use crate::facade::{
    BridgeAsyncSourceDeclarationDraft, BridgeAsyncSourceDeclarationRejection,
    LoweredBridgeAsyncSourceDeclaration,
};

use super::super::BridgeOwnedSignalRuntime;
use super::BridgeOwnedAsyncRequestResponseDeclaration;

impl BridgeOwnedSignalRuntime {
    pub(in crate::conditional_execution) fn install_owned_async_request_response(
        &mut self,
        declaration: BridgeOwnedAsyncRequestResponseDeclaration,
    ) -> Result<LoweredBridgeAsyncSourceDeclaration, BridgeAsyncSourceDeclarationRejection> {
        if self
            .async_declarations
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .contains_key(&declaration.identity)
        {
            return Err(BridgeAsyncSourceDeclarationRejection::new(
                crate::facade::BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                "owned async declaration identity is already installed",
            ));
        }
        let node = self.signal_runtime_mut().graph_mut().node().build();
        let retry_delay = worth_signal::facade::TemporalDuration::temporal_duration(
            declaration.retry_delay_ticks,
        )
        .map_err(|error| {
            BridgeAsyncSourceDeclarationRejection::new(
                crate::facade::BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                error.to_string(),
            )
        })?;
        let timeout =
            worth_signal::facade::TemporalDuration::temporal_duration(declaration.timeout_ticks)
                .map_err(|error| {
                    BridgeAsyncSourceDeclarationRejection::new(
                crate::facade::BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                error.to_string(),
            )
                })?;
        let resource = worth_signal::facade::ResourceNodeDeclaration::new(
            worth_signal::facade::ResourceNodeId::from_node(node),
            worth_signal::facade::ResourcePayloadContract::new(
                worth_signal::facade::ResourcePayloadContractId::new(declaration.payload_contract),
            )
            .with_max_payload_bytes(declaration.max_payload_bytes),
        )
        .with_observation_policy(
            worth_signal::facade::ResourceObservationPolicyDeclaration::LifecycleOnly,
        )
        .with_retry_policy(
            worth_signal::facade::ResourceRetryPolicyDeclaration::FixedDelay { delay: retry_delay },
        )
        .with_timeout_policy(
            worth_signal::facade::ResourceTimeoutPolicyDeclaration::PerAttemptTimeout { timeout },
        )
        .with_retry_max_attempts(declaration.retry_max_attempts);
        let draft = BridgeAsyncSourceDeclarationDraft::request_response(
            crate::facade::BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(
                declaration.identity.clone(),
            ),
            crate::facade::BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(format!(
                "owned-async-internal:{}",
                declaration.identity
            )),
            resource.clone(),
        );
        let install = (|| {
            let validated = self.bridge.validate_async_source_declaration(draft)?;
            self.signal_runtime_mut()
                .declare_resource_node(resource.clone())
                .map_err(|error| {
                    BridgeAsyncSourceDeclarationRejection::new(
                        crate::facade::BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                        error.to_string(),
                    )
                })?;
            let descriptor = self
                .signal_runtime_mut()
                .resource_descriptor_for_node(resource.node())
                .cloned()
                .ok_or_else(|| {
                    BridgeAsyncSourceDeclarationRejection::new(
                        crate::facade::BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                        "owned Signal runtime lost its installed resource descriptor",
                    )
                })?;
            Ok(
                crate::facade::LoweredBridgeAsyncSourceDeclaration::from_request_response_parts(
                    &validated, resource, descriptor,
                ),
            )
        })();
        let lowered = match install {
            Ok(lowered) => lowered,
            Err(denial) => {
                let _ = self.signal_runtime_mut().graph_mut().unregister_node(node);
                return Err(denial);
            }
        };
        self.async_declarations
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(declaration.identity, lowered.clone());
        Ok(lowered)
    }

    pub fn installed_owned_async_declaration(
        &self,
        identity: &str,
    ) -> Option<LoweredBridgeAsyncSourceDeclaration> {
        self.async_declarations
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(identity)
            .cloned()
    }

    pub fn owned_async_declaration_count(&self) -> usize {
        self.async_declarations
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}
