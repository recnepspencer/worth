use crate::scenario::BlobHarnessScenarioMetadata;
use crate::{PhysicalScenarioCanonicalIdentity, PhysicalSimulationScenarioFamily};
use worth_store_blob_chunks::BlobHarnessChunkTopology;

use super::{
    PhysicalSimulationPlanIdentity, PhysicalSimulationProfile, RequiredActorSet,
    RequiredFixtureClassSet, RequiredObserverSet, RequiredOracleFamilySet,
    RequiredPhysicalDriverSet, SimulationEvidencePolicy, SimulationPlanDenial,
};
use crate::{
    AdmittedDriverContractSet, ForbiddenShortcutSet, PhysicalResourceEnvelope,
    PhysicalSimulationCapabilitySet, RequiredCounterContractSet, YieldpointScheduleBinding,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalSimulationPlan {
    scenario_identity: PhysicalScenarioCanonicalIdentity,
    scenario_family: PhysicalSimulationScenarioFamily,
    identity: PhysicalSimulationPlanIdentity,
    profile: PhysicalSimulationProfile,
    resource_envelope: PhysicalResourceEnvelope,
    required_capabilities: PhysicalSimulationCapabilitySet,
    actors: RequiredActorSet,
    drivers: RequiredPhysicalDriverSet,
    driver_contracts: AdmittedDriverContractSet,
    yieldpoint_binding: YieldpointScheduleBinding,
    observers: RequiredObserverSet,
    oracle_families: RequiredOracleFamilySet,
    counter_contracts: RequiredCounterContractSet,
    fixture_classes: RequiredFixtureClassSet,
    evidence_policy: SimulationEvidencePolicy,
    forbidden_shortcuts: ForbiddenShortcutSet,
    blob_harness_metadata: Option<BlobHarnessScenarioMetadata>,
    blob_harness_topology: Option<BlobHarnessChunkTopology>,
}

impl PhysicalSimulationPlan {
    pub(crate) fn from_parts(
        parts: PhysicalSimulationPlanParts,
    ) -> Result<Self, SimulationPlanDenial> {
        let identity = PhysicalSimulationPlanIdentity::from_parts(&parts)?;
        Ok(Self {
            scenario_identity: parts.scenario_identity,
            scenario_family: parts.scenario_family,
            identity,
            profile: parts.profile,
            resource_envelope: parts.resource_envelope,
            required_capabilities: parts.required_capabilities,
            actors: parts.actors,
            drivers: parts.drivers,
            driver_contracts: parts.driver_contracts,
            yieldpoint_binding: parts.yieldpoint_binding,
            observers: parts.observers,
            oracle_families: parts.oracle_families,
            counter_contracts: parts.counter_contracts,
            fixture_classes: parts.fixture_classes,
            evidence_policy: parts.evidence_policy,
            forbidden_shortcuts: parts.forbidden_shortcuts,
            blob_harness_metadata: parts.blob_harness_metadata,
            blob_harness_topology: parts.blob_harness_topology,
        })
    }

    pub const fn scenario_identity(&self) -> &PhysicalScenarioCanonicalIdentity {
        &self.scenario_identity
    }

    pub const fn scenario_family(&self) -> PhysicalSimulationScenarioFamily {
        self.scenario_family
    }

    pub const fn identity(&self) -> &PhysicalSimulationPlanIdentity {
        &self.identity
    }

    pub const fn profile(&self) -> PhysicalSimulationProfile {
        self.profile
    }

    pub const fn resource_envelope(&self) -> PhysicalResourceEnvelope {
        self.resource_envelope
    }

    pub const fn required_capabilities(&self) -> &PhysicalSimulationCapabilitySet {
        &self.required_capabilities
    }

    pub const fn actors(&self) -> &RequiredActorSet {
        &self.actors
    }

    pub const fn drivers(&self) -> &RequiredPhysicalDriverSet {
        &self.drivers
    }

    pub const fn driver_contracts(&self) -> &AdmittedDriverContractSet {
        &self.driver_contracts
    }

    pub const fn yieldpoint_binding(&self) -> &YieldpointScheduleBinding {
        &self.yieldpoint_binding
    }

    pub const fn observers(&self) -> &RequiredObserverSet {
        &self.observers
    }

    pub const fn oracle_families(&self) -> &RequiredOracleFamilySet {
        &self.oracle_families
    }

    pub const fn counter_contracts(&self) -> &RequiredCounterContractSet {
        &self.counter_contracts
    }

    pub const fn fixture_classes(&self) -> &RequiredFixtureClassSet {
        &self.fixture_classes
    }

    pub const fn evidence_policy(&self) -> SimulationEvidencePolicy {
        self.evidence_policy
    }

    pub const fn forbidden_shortcuts(&self) -> &ForbiddenShortcutSet {
        &self.forbidden_shortcuts
    }

    pub const fn blob_harness_metadata(&self) -> Option<BlobHarnessScenarioMetadata> {
        self.blob_harness_metadata
    }

    pub const fn blob_harness_topology(&self) -> Option<BlobHarnessChunkTopology> {
        self.blob_harness_topology
    }
}

pub const fn require_lowered_physical_simulation_plan(
    plan: &PhysicalSimulationPlan,
) -> &PhysicalSimulationPlan {
    plan
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhysicalSimulationPlanParts {
    pub(crate) scenario_identity: PhysicalScenarioCanonicalIdentity,
    pub(crate) scenario_family: PhysicalSimulationScenarioFamily,
    pub(crate) profile: PhysicalSimulationProfile,
    pub(crate) resource_envelope: PhysicalResourceEnvelope,
    pub(crate) required_capabilities: PhysicalSimulationCapabilitySet,
    pub(crate) actors: RequiredActorSet,
    pub(crate) drivers: RequiredPhysicalDriverSet,
    pub(crate) driver_contracts: AdmittedDriverContractSet,
    pub(crate) yieldpoint_binding: YieldpointScheduleBinding,
    pub(crate) observers: RequiredObserverSet,
    pub(crate) oracle_families: RequiredOracleFamilySet,
    pub(crate) counter_contracts: RequiredCounterContractSet,
    pub(crate) fixture_classes: RequiredFixtureClassSet,
    pub(crate) evidence_policy: SimulationEvidencePolicy,
    pub(crate) forbidden_shortcuts: ForbiddenShortcutSet,
    pub(crate) blob_harness_metadata: Option<BlobHarnessScenarioMetadata>,
    pub(crate) blob_harness_topology: Option<BlobHarnessChunkTopology>,
}
