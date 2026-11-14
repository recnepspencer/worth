//! Borrow the actual branch's derived execution state without transferring owner authority.
use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};
use std::sync::{Arc, RwLock};

pub struct BridgeConditionalExecutionView<'a> {
    pub(super) async_definitions: Option<
        &'a std::collections::BTreeMap<
            Arc<str>,
            super::owned_async::definition_source::OwnedAsyncDefinitionSource,
        >,
    >,
    pub(super) async_declarations: &'a RwLock<
        std::collections::BTreeMap<
            Arc<str>,
            super::owned_async::BridgeInstalledOwnedAsyncDeclaration,
        >,
    >,
    pub(super) async_observation_authority: &'a Arc<()>,
    pub(super) signal_services: Option<&'a super::service_binding::BridgeSignalServiceBinding>,
    pub(super) correspondence_allocations:
        &'a crate::correspondence::SharedCorrespondenceAllocationRegistry,
    pub(super) bridge: &'a crate::facade::RuntimeBridge,
    pub(super) managed_clock_lanes: &'a std::sync::Mutex<
        std::collections::BTreeMap<
            Arc<str>,
            Arc<std::sync::Mutex<super::managed_time::BridgeManagedClockLane>>,
        >,
    >,
    pub(super) declaration_scope: Option<(
        &'a worth_proof::BridgeProgramInstallationBinding,
        &'a Arc<RwLock<super::authored_declaration::DeclarationIndex>>,
    )>,
    pub(super) retention: &'a Arc<super::retention::BridgeRetentionLedger>,
    pub(super) conditional_lowerings:
        &'a Arc<RwLock<super::lowering_registry::BridgeConditionalLoweringRegistry>>,
    pub(super) signal_graph_instance_id: u64,
    pub(super) conditional_evaluation_budget:
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    pub(super) source_owner: Option<&'a worth_proof::ConditionalSourceObservationOwner>,
    pub(super) durable_owner:
        Option<&'a worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()>>,
}
impl BridgeOwnedSignalRuntime {
    pub(super) fn execution_view(&self) -> BridgeConditionalExecutionView<'_> {
        BridgeConditionalExecutionView {
            async_definitions: None,
            async_declarations: &self.async_declarations,
            async_observation_authority: &self.async_observation_authority,
            signal_services: self.signal_services.as_ref(),
            bridge: &self.bridge,
            correspondence_allocations: &self.bridge.correspondence_allocations,
            managed_clock_lanes: &self.managed_clock_lanes,
            declaration_scope: None,
            retention: &self.retention,
            conditional_lowerings: &self.conditional_lowerings,
            signal_graph_instance_id: self.signal_graph_instance_id,
            conditional_evaluation_budget: self.conditional_evaluation_budget,
            source_owner: self
                .signal_services
                .as_ref()
                .map(|services| services.source_owner()),
            durable_owner: None,
        }
    }
}
impl BridgeConditionalExecutionView<'_> {
    pub const fn conditional_evaluation_budget(
        &self,
    ) -> worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
        self.conditional_evaluation_budget
    }

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

impl BridgeConditionalExecutionView<'_> {
    pub fn admit_declaration_signal_basis(
        &self,
        declaration: &super::BridgeAuthoredConditionalDeclaration,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<super::BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        let (installation, declarations) = self.declaration_scope.ok_or_else(|| {
            BridgeConditionalDenial::operation_route_refused(
                "authored declaration requires its authored receiver",
            )
        })?;
        if installation.admit(&declaration.installation).is_err() {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::StaleLowering,
                "conditional declaration belongs to another installed programme",
            ));
        }
        let lowering = declarations
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&declaration.key)
            .cloned()
            .ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::StaleLowering,
                    "conditional declaration has no recovered branch definition",
                )
            })?;
        self.admit_exact_conditional_signal_basis(&lowering, basis)
    }
}
