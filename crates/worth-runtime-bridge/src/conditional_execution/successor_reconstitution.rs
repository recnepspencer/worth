use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind,
    BridgeConditionalRuntimeReconstitutionReport, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};
use worth_signal::facade::branch::{AdmittedSignalBranchBasis, SignalConditionalExecutionPort};

mod lowering;
mod managed_clocks;

/// Complete derived candidate under the incumbent Signal owner. Dropping it
/// releases only candidate service/clock custody and leaves the live generation.
pub struct BridgePreparedConditionalReconstitution {
    runtime_key: u64,
    retention: Arc<super::retention::BridgeRetentionLedger>,
    bridge: crate::facade::RuntimeBridge,
    predecessor: HashMap<
        super::lowering_registry::BridgeConditionalLoweringKey,
        Arc<BridgeInstalledConditionalLowering>,
    >,
    lowerings: HashMap<
        super::lowering_registry::BridgeConditionalLoweringKey,
        Arc<BridgeInstalledConditionalLowering>,
    >,
    managed_clocks: BTreeMap<Arc<str>, Arc<Mutex<super::managed_time::BridgeManagedClockLane>>>,
    service: Arc<SignalConditionalExecutionPort<(), (), ()>>,
    report: BridgeConditionalRuntimeReconstitutionReport,
}

impl BridgeOwnedSignalRuntime {
    pub fn prepare_conditional_reconstitution(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<BridgePreparedConditionalReconstitution, BridgeConditionalDenial> {
        let services = self.signal_services()?;
        let (service, signal) = services
            .conditional_extension_port()
            .reconstitute_at_basis(basis)
            .map_err(|denial| {
                reconstitution_denial(format!("Signal service readmission was denied: {denial:?}"))
            })?;
        let mut bridge = self.bridge.clone();
        bridge.semantic_dependency_registry = bridge
            .semantic_dependency_registry
            .reconstruct_derived_indexes()
            .map_err(|error| {
                reconstitution_denial(format!("semantic index reconstruction failed: {error:?}"))
            })?;
        bridge.aspect_registry = bridge
            .aspect_registry
            .reconstruct_derived_indexes()
            .map_err(|error| {
                reconstitution_denial(format!("aspect index reconstruction failed: {error:?}"))
            })?;
        let allocations = bridge
            .correspondence_allocations
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reconstruct_derived_indexes();
        bridge.correspondence_allocations = Arc::new(RwLock::new(allocations));
        let correspondence = bridge
            .rebuild_correspondence_allocation_index()
            .map_err(|error| {
                reconstitution_denial(format!("correspondence reconstruction failed: {error:?}"))
            })?;
        let predecessor = self
            .conditional_lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot();
        let service = Arc::new(service);
        let lowerings = predecessor
            .iter()
            .map(|(key, installed)| {
                lowering::readmit(installed, &service).map(|readmitted| (key.clone(), readmitted))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        let report = BridgeConditionalRuntimeReconstitutionReport::new(
            signal,
            correspondence,
            self.signal_graph_instance_id,
            lowerings.len(),
        );
        Ok(BridgePreparedConditionalReconstitution {
            runtime_key: self.bridge.signal_runtime_key,
            retention: Arc::clone(&self.retention),
            bridge,
            predecessor,
            lowerings,
            managed_clocks: BTreeMap::new(),
            service,
            report,
        })
    }

    /// Exclusive derived-generation activation. Preparation has completed all
    /// reconstruction; final exact-basis validation precedes routing and lease replacement.
    pub fn activate_conditional_reconstitution(
        &mut self,
        mut candidate: BridgePreparedConditionalReconstitution,
    ) -> Result<BridgeConditionalRuntimeReconstitutionReport, BridgeConditionalDenial> {
        candidate.service.active_node_count().map_err(|denial| {
            reconstitution_denial(format!(
                "prepared Signal service is no longer current: {denial:?}"
            ))
        })?;
        let mut installed = self
            .conditional_lowerings
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if candidate.runtime_key != self.bridge.signal_runtime_key
            || installed.len() != candidate.predecessor.len()
            || installed.values().any(|current| {
                let key = super::contract::lowering_key(current);
                !candidate
                    .predecessor
                    .get(&key)
                    .is_some_and(|expected| Arc::ptr_eq(current, expected))
            })
        {
            return Err(reconstitution_denial(
                "conditional generation changed during reconstruction",
            ));
        }
        for lowering in installed.values() {
            lowering.lease.revoke_liveness();
        }
        let clocks = self
            .managed_clock_lanes
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for clock in clocks.values() {
            clock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .revoke_liveness();
        }
        let allocations = candidate
            .bridge
            .correspondence_allocations
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        *self
            .bridge
            .correspondence_allocations
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = allocations;
        candidate.bridge.correspondence_allocations =
            Arc::clone(&self.bridge.correspondence_allocations);
        self.bridge = candidate.bridge;
        let lowerings = candidate.lowerings;
        installed.replace_installed(lowerings);
        *clocks = candidate.managed_clocks;
        let mut targets = self
            .owned_conditional_targets
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *targets = Default::default();
        for lowering in installed.values() {
            targets.register(lowering);
        }
        self.signal_services()?
            .publish_reconstituted_service(candidate.service);
        Ok(candidate.report)
    }

    #[cfg(test)]
    pub(crate) fn destroy_reconstitutable_indexes_for_test(&mut self) {
        self.bridge
            .semantic_dependency_registry
            .destroy_derived_indexes();
        self.bridge.aspect_registry.destroy_derived_indexes();
        self.bridge
            .correspondence_allocations
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .destroy_derived_indexes();
    }
}

impl BridgePreparedConditionalReconstitution {
    pub fn readmit_lowering(
        &self,
        predecessor: &Arc<BridgeInstalledConditionalLowering>,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeConditionalDenial> {
        let key = super::contract::lowering_key(predecessor);
        if !self
            .predecessor
            .get(&key)
            .is_some_and(|installed| Arc::ptr_eq(installed, predecessor))
        {
            return Err(reconstitution_denial(
                "reconstitution requires the exact incumbent lowering",
            ));
        }
        self.lowerings
            .get(&key)
            .cloned()
            .ok_or_else(|| reconstitution_denial("reconstituted lowering is absent"))
    }

    /// Retains the candidate generation's exact Signal service binding before
    /// activation so reconstructed Query state cannot fall back to ambient
    /// Signal currentness after the generation swap.
    pub fn admit_exact_conditional_signal_basis(
        &self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<super::BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        let key = super::contract::lowering_key(lowering);
        if !self
            .lowerings
            .get(&key)
            .is_some_and(|installed| Arc::ptr_eq(installed, lowering))
        {
            return Err(reconstitution_denial(
                "Signal basis admission requires the exact reconstructed lowering",
            ));
        }
        let signal_port = lowering
            .signal_port()
            .ok_or_else(|| reconstitution_denial("reconstructed lowering has no Signal port"))?;
        if signal_port.issuance_basis().admission_identity() != basis.admission_identity() {
            return Err(reconstitution_denial(
                "reconstructed lowering belongs to another Signal basis",
            ));
        }
        Ok(super::BridgeConditionalSignalBasisBinding {
            lowering: Arc::clone(lowering),
            signal_port,
        })
    }
}

fn reconstitution_denial(detail: impl Into<String>) -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(
        BridgeConditionalDenialKind::SignalContractInstallation,
        detail,
    )
}
