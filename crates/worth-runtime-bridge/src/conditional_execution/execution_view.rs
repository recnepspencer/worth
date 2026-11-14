//! Borrow the actual branch's derived execution state without transferring owner authority.
use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};
use std::sync::{Arc, RwLock};

pub(super) struct BridgeConditionalExecutionView<'a> {
    pub(super) bridge: &'a crate::facade::RuntimeBridge,
    pub(super) allocations: &'a crate::correspondence::SharedCorrespondenceAllocationRegistry,
    pub(super) retention: &'a Arc<super::retention::BridgeRetentionLedger>,
    pub(super) conditional_lowerings:
        &'a Arc<RwLock<super::lowering_registry::BridgeConditionalLoweringRegistry>>,
    pub(super) signal_graph_instance_id: u64,
    pub(super) source_owner: Option<&'a worth_proof::ConditionalSourceObservationOwner>,
    pub(super) durable_owner:
        Option<&'a worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()>>,
}
impl BridgeOwnedSignalRuntime {
    pub(super) fn execution_view(&self) -> BridgeConditionalExecutionView<'_> {
        BridgeConditionalExecutionView {
            bridge: &self.bridge,
            allocations: &self.bridge.correspondence_allocations,
            retention: &self.retention,
            conditional_lowerings: &self.conditional_lowerings,
            signal_graph_instance_id: self.signal_graph_instance_id,
            source_owner: self
                .signal_services
                .as_ref()
                .map(|services| services.source_owner()),
            durable_owner: None,
        }
    }
}
impl BridgeConditionalExecutionView<'_> {
    pub(super) fn source_owner(
        &self,
    ) -> Result<&worth_proof::ConditionalSourceObservationOwner, BridgeConditionalDenial> {
        self.source_owner.ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution(
                    super::BridgeSignalDenial::UnsealedOwnerServices,
                ),
                "conditional execution requires its original sealed source authority",
            )
        })
    }
    pub(super) fn conditional_port(
        &self,
        lowering: &BridgeInstalledConditionalLowering,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<super::signal_port::BridgeConditionalSignalPort, BridgeConditionalDenial> {
        if let Some(port) = lowering.signal_port() {
            return Ok(port);
        }
        let owner = self.durable_owner.ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution(
                    super::BridgeSignalDenial::MissingConditionalPort,
                ),
                "installed lowering has no exact Signal service binding",
            )
        })?;
        let port = owner
            .issue_conditional_execution_service(
                basis,
                &self.bridge.signal_aspect_lowering_owner,
                &self.source_owner()?.authority(),
            )
            .map_err(|cause| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SignalExecution(
                        super::BridgeSignalDenial::ConditionalServiceIssuance(cause),
                    ),
                    "durable Signal refused the actual conditional branch basis",
                )
            })?;
        port.validate_installed_contract(lowering.signal_contract())
            .map_err(|cause| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::StaleLowering,
                    format!("durable branch rejected the installed definition: {cause:?}"),
                )
            })?;
        let port = Arc::new(port);
        lowering.bind_signal_port(Arc::clone(&port));
        // A concurrent first demand may have installed its same-owner port.
        // Reuse the retained original rather than discarding its evaluation identity sequence.
        lowering.signal_port().ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::StaleLowering,
                "conditional lowering lost its issued port",
            )
        })
    }
}
