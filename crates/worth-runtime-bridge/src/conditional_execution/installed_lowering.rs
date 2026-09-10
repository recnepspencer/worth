use std::sync::Arc;

use worth_signal::facade::InstalledSignalConditionalContract;

use super::{BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeConditionalProviderSet};
use crate::correspondence::BridgeInstalledSemanticCorrespondence;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BridgeInstalledConditionalLoweringCounters {
    pub contract_admission_checks: usize,
    pub correspondence_registrations_inspected: usize,
    pub correspondence_targets_inspected: usize,
    pub signal_graph_checks: usize,
    pub signal_node_ownership_checks: usize,
    pub dependency_registry_compilations: usize,
    pub dependency_registry_existing_key_lookups: usize,
    pub dependency_registry_batch_key_lookups: usize,
    pub dependency_registry_commits: usize,
    pub unrelated_dependency_registry_scans: usize,
    pub semantic_observation_plan_compilations: usize,
    pub signal_node_admissions: usize,
    pub correspondence_batch_preparations: usize,
    pub signal_contract_lowerings: usize,
    pub correspondence_admissions: usize,
    pub signal_targets_joined: usize,
    pub provider_checks: usize,
    pub signal_contract_installations: usize,
}

pub struct BridgeInstalledConditionalLowering {
    pub(crate) bridge_runtime_key: u64,
    pub(super) registry_key: super::lowering_registry::BridgeConditionalLoweringKey,
    pub(super) _authority:
        super::lowering_authority::BridgeInstalledConditionalLoweringAuthorityIdentity,
    pub(crate) projection: super::BridgeConditionalLoweringProjectionIdentity,
    pub(crate) contract: super::BridgeConditionalContract,
    pub(crate) location: super::BridgeConditionalLocation,
    pub(crate) correspondences: Arc<[BridgeInstalledSemanticCorrespondence]>,
    pub(super) semantic_observation_plan:
        Option<super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan>,
    pub(crate) signal_contract: std::sync::OnceLock<InstalledSignalConditionalContract>,
    pub(super) signal_port: std::sync::OnceLock<super::signal_port::BridgeConditionalSignalPort>,
    pub(super) definition_custody:
        std::sync::OnceLock<worth_signal::facade::branch::SignalConditionalInstallationCustody>,
    pub(crate) providers: BridgeConditionalProviderSet,
    pub(super) provider_admission: super::provider_admission::BridgeConditionalProviderAdmission,
    pub(super) lease: Arc<super::liveness::BridgeConditionalLoweringLease>,
    pub(super) _definition_retention: Option<Arc<super::retention::BridgeRetentionReservation>>,
    pub(crate) counters: BridgeInstalledConditionalLoweringCounters,
}

impl BridgeInstalledConditionalLowering {
    pub fn projection(&self) -> &super::BridgeConditionalLoweringProjectionIdentity {
        &self.projection
    }
    pub fn location(&self) -> &super::BridgeConditionalLocation {
        &self.location
    }
    pub fn contract(&self) -> &super::BridgeConditionalContract {
        &self.contract
    }
    pub fn signal_graph_instance_id(&self) -> u64 {
        self.signal_contract().graph_instance_id()
    }
    pub fn signal_node(&self) -> worth_signal::facade::NodeId {
        self.signal_contract().node()
    }
    pub fn signal_definition_generation(&self) -> u64 {
        self.signal_contract().generation()
    }
    pub fn signal_basis_admission_identity(
        &self,
    ) -> Option<worth_signal::facade::branch::SignalBranchBasisAdmissionIdentity> {
        self.signal_port()
            .map(|port| port.issuance_basis().admission_identity().clone())
    }
    pub(super) fn bind_signal_port(
        &self,
        port: Arc<worth_signal::facade::branch::SignalConditionalExecutionPort<(), (), ()>>,
    ) {
        self.signal_port
            .set(super::signal_port::BridgeConditionalSignalPort::shared(
                port,
            ))
            .ok();
    }

    pub(super) fn reserve_signal_port(&self) {
        self.signal_port
            .set(super::signal_port::BridgeConditionalSignalPort::reserve())
            .ok();
    }

    pub(super) fn initialize_reserved_signal_port(
        &self,
        port: worth_signal::facade::branch::SignalConditionalExecutionPort<(), (), ()>,
    ) {
        if let Some(port_slot) = self.signal_port.get() {
            port_slot.initialize(port);
        }
    }

    pub(super) fn initialize_signal_contract(&self, contract: InstalledSignalConditionalContract) {
        self.signal_contract.get_or_init(|| contract);
    }

    pub(super) fn initialize_definition_custody(
        &self,
        custody: worth_signal::facade::branch::SignalConditionalInstallationCustody,
    ) {
        self.definition_custody.get_or_init(|| custody);
    }

    pub(super) fn signal_port(&self) -> Option<super::signal_port::BridgeConditionalSignalPort> {
        self.signal_port.get().cloned()
    }

    pub(super) fn signal_contract(&self) -> &InstalledSignalConditionalContract {
        self.signal_contract
            .get()
            .expect("an installed lowering retains its Signal contract")
    }
    pub fn counters(&self) -> BridgeInstalledConditionalLoweringCounters {
        self.counters
    }
    pub fn correspondence_count(&self) -> usize {
        self.correspondences.len()
    }

    pub(super) fn signal_dependency_aspects(&self) -> worth_signal::facade::AspectMask {
        self.signal_aspects_for(|_| true)
    }

    pub(super) fn signal_condition_aspects(
        &self,
        dependency_ordinals: &[usize],
    ) -> worth_signal::facade::AspectMask {
        self.signal_aspects_for(|dependency| {
            dependency_ordinals.contains(&dependency.dependency_ordinal())
        })
    }

    fn signal_aspects_for(
        &self,
        include: impl Fn(&crate::correspondence::BridgeSemanticDependencyCandidate) -> bool,
    ) -> worth_signal::facade::AspectMask {
        let mut mask = worth_signal::facade::AspectMask::EMPTY;
        for correspondence in self.correspondences.iter() {
            if include(correspondence.dependency()) {
                for target in correspondence.targets.as_slice() {
                    mask.insert(target.aspect);
                }
            }
        }
        mask
    }

    /// Installed semantic dependencies retained by this lowering.
    ///
    /// Consumers may use these declarations to narrow their own candidate
    /// selection. The declarations remain non-authoritative without this
    /// current installed lowering.
    pub fn semantic_dependencies(
        &self,
    ) -> impl ExactSizeIterator<Item = &crate::correspondence::BridgeSemanticDependencyCandidate>
    {
        self.correspondences
            .iter()
            .map(|correspondence| correspondence.dependency())
    }

    pub fn dependency_locality(
        &self,
        ordinal: usize,
    ) -> Option<&crate::correspondence::BridgeSemanticLocality> {
        self.correspondences
            .iter()
            .find(|correspondence| correspondence.dependency().dependency_ordinal() == ordinal)
            .map(|correspondence| correspondence.dependency().locality())
    }

    pub fn validate_signal_decision_contract(
        &self,
        evidence: &worth_signal::facade::SignalConditionalDecisionEvidence,
    ) -> Result<(), BridgeConditionalDenial> {
        if !self.signal_contract().retains_decision(evidence) {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalContractMismatch,
                "Signal decision evidence does not retain the installed conditional contract",
            ));
        }
        Ok(())
    }
}
