use std::sync::PoisonError;

use crate::facade::{
    BridgeAsyncSourceDeclarationDraft, BridgeAsyncSourceDeclarationRejection,
    LoweredBridgeAsyncSourceDeclaration,
};

use super::super::BridgeOwnedSignalRuntime;
use super::BridgeOwnedAsyncRequestResponseDeclaration;

pub(in crate::conditional_execution) struct BridgeInstalledOwnedAsyncDeclaration {
    pub(super) lowered: LoweredBridgeAsyncSourceDeclaration,
    pub(in crate::conditional_execution) source:
        super::definition_source::OwnedAsyncDefinitionSource,
}

impl BridgeOwnedSignalRuntime {
    pub(in crate::conditional_execution) fn install_owned_async_request_response(
        &mut self,
        declaration: BridgeOwnedAsyncRequestResponseDeclaration,
    ) -> Result<LoweredBridgeAsyncSourceDeclaration, BridgeAsyncSourceDeclarationRejection> {
        if declaration.process_local_retention && declaration.managed_queue_capacity.is_none() {
            return Err(BridgeAsyncSourceDeclarationRejection::new(
                crate::facade::BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                "process-local resource retention requires an installed managed queue",
            ));
        }
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
        let description = declaration.description();
        let node = self.signal_runtime_mut().graph_mut().node().build();
        let resource = declaration.signal_declaration(node)?;
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
            .insert(
                declaration.identity,
                BridgeInstalledOwnedAsyncDeclaration {
                    lowered: lowered.clone(),
                    source: super::definition_source::OwnedAsyncDefinitionSource {
                        node,
                        description,
                    },
                },
            );
        Ok(lowered)
    }

    pub fn owned_async_declaration_count(&self) -> usize {
        self.execution_view().owned_async_declaration_count()
    }
}
