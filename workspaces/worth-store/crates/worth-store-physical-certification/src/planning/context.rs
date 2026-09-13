use super::{
    ForbiddenShortcutSet, PhysicalSimulationCapabilitySet, PhysicalSimulationProfile,
    PhysicalSimulationProfileSet, SimulationEvidencePolicy, SupportedObserverSet,
    SupportedOracleFamilySet, SupportedPhysicalDriverSet,
};
use crate::{AdmittedDriverContractSet, PhysicalResourceEnvelope};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationPlanningContext {
    profile: PhysicalSimulationProfile,
    resource_envelope: PhysicalResourceEnvelope,
    supported_profiles: PhysicalSimulationProfileSet,
    capabilities: PhysicalSimulationCapabilitySet,
    driver_contracts: AdmittedDriverContractSet,
    supported_drivers: SupportedPhysicalDriverSet,
    supported_observers: SupportedObserverSet,
    supported_oracle_families: SupportedOracleFamilySet,
    evidence_policy: Option<SimulationEvidencePolicy>,
    forbidden_shortcuts: Option<ForbiddenShortcutSet>,
}

impl SimulationPlanningContext {
    pub fn developer_smoke() -> Self {
        Self::for_profile(PhysicalSimulationProfile::DeveloperSmoke)
    }

    pub fn for_profile(profile: PhysicalSimulationProfile) -> Self {
        Self {
            profile,
            resource_envelope: PhysicalResourceEnvelope::for_profile(profile),
            supported_profiles: PhysicalSimulationProfileSet::developer_smoke_only(),
            capabilities: PhysicalSimulationCapabilitySet::empty(),
            driver_contracts: AdmittedDriverContractSet::empty(),
            supported_drivers: SupportedPhysicalDriverSet::empty(),
            supported_observers: SupportedObserverSet::empty(),
            supported_oracle_families: SupportedOracleFamilySet::empty(),
            evidence_policy: None,
            forbidden_shortcuts: None,
        }
    }

    pub fn with_supported_profiles(
        mut self,
        supported_profiles: PhysicalSimulationProfileSet,
    ) -> Self {
        self.supported_profiles = supported_profiles;
        self
    }

    pub fn with_resource_envelope(mut self, resource_envelope: PhysicalResourceEnvelope) -> Self {
        self.resource_envelope = resource_envelope;
        self
    }

    pub fn with_capabilities(mut self, capabilities: PhysicalSimulationCapabilitySet) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_driver_contracts(mut self, driver_contracts: AdmittedDriverContractSet) -> Self {
        self.driver_contracts = driver_contracts;
        self
    }

    pub fn with_supported_drivers(mut self, supported_drivers: SupportedPhysicalDriverSet) -> Self {
        self.supported_drivers = supported_drivers;
        self
    }

    pub fn with_supported_observers(mut self, supported_observers: SupportedObserverSet) -> Self {
        self.supported_observers = supported_observers;
        self
    }

    pub fn with_supported_oracle_families(
        mut self,
        supported_oracle_families: SupportedOracleFamilySet,
    ) -> Self {
        self.supported_oracle_families = supported_oracle_families;
        self
    }

    pub fn with_evidence_policy(mut self, evidence_policy: SimulationEvidencePolicy) -> Self {
        self.evidence_policy = Some(evidence_policy);
        self
    }

    pub fn with_forbidden_shortcuts(mut self, forbidden_shortcuts: ForbiddenShortcutSet) -> Self {
        self.forbidden_shortcuts = Some(forbidden_shortcuts);
        self
    }

    pub const fn profile(&self) -> PhysicalSimulationProfile {
        self.profile
    }

    pub const fn resource_envelope(&self) -> PhysicalResourceEnvelope {
        self.resource_envelope
    }

    pub const fn capabilities(&self) -> &PhysicalSimulationCapabilitySet {
        &self.capabilities
    }

    pub const fn driver_contracts(&self) -> &AdmittedDriverContractSet {
        &self.driver_contracts
    }

    pub const fn supported_drivers(&self) -> &SupportedPhysicalDriverSet {
        &self.supported_drivers
    }

    pub const fn supported_observers(&self) -> &SupportedObserverSet {
        &self.supported_observers
    }

    pub const fn supported_oracle_families(&self) -> &SupportedOracleFamilySet {
        &self.supported_oracle_families
    }

    pub const fn evidence_policy(&self) -> Option<SimulationEvidencePolicy> {
        self.evidence_policy
    }

    pub const fn forbidden_shortcuts(&self) -> Option<&ForbiddenShortcutSet> {
        self.forbidden_shortcuts.as_ref()
    }

    pub const fn supported_profiles(&self) -> &PhysicalSimulationProfileSet {
        &self.supported_profiles
    }
}
