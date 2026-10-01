use super::capability::StrategyCapability;
use super::counter_planning::declared_strategy_counter_envelope;
use super::key_law_validation::DeclaredKeyLawPosture;
use super::posture::{
    StrategyAmplificationProfile, StrategyCorruptionIsolationBehavior, StrategyLocalityProfile,
    StrategyMaterializationPosture, StrategyRebuildSourceRequirement,
};
use super::{LayoutStrategyFamily, StrategyAuthorityBasis};
use crate::access::budget::PlannedCounterEnvelope;
use crate::catalog::{
    ArtifactFamilyAccessLane, ArtifactFamilyAuthorityClass, ArtifactFamilyLifecycleAdmission,
    DurableArtifactMigrationPosture, DurableArtifactProjectionClass, DurableArtifactRebuildPosture,
};
use crate::keyspace::{
    CanonicalKeyEncoding, ComparatorLaw, PhysicalKeyDomainWitness, PrefixLawWitness,
    RangeBoundLawWitness,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StrategyDeclaration {
    authority_basis: StrategyAuthorityBasis,
    family: LayoutStrategyFamily,
    capability: StrategyCapability,
    locality: StrategyLocalityProfile,
    amplification: StrategyAmplificationProfile,
    materialization: StrategyMaterializationPosture,
    rebuild_source: StrategyRebuildSourceRequirement,
    corruption_isolation: StrategyCorruptionIsolationBehavior,
    access_lane: ArtifactFamilyAccessLane,
    authority_class: ArtifactFamilyAuthorityClass,
    rebuild_posture: DurableArtifactRebuildPosture,
    migration_posture: DurableArtifactMigrationPosture,
    projection_classes: &'static [DurableArtifactProjectionClass],
    canonical_key_encoding: Option<CanonicalKeyEncoding>,
    comparator_law: Option<ComparatorLaw>,
    prefix_law: Option<PrefixLawWitness>,
    range_bound_law: Option<RangeBoundLawWitness>,
    planned_counter_envelope: Option<PlannedCounterEnvelope>,
}

impl StrategyDeclaration {
    pub(super) const fn btree_range(
        authority_basis: StrategyAuthorityBasis,
        key_laws: DeclaredKeyLawPosture,
    ) -> Self {
        let lifecycle = authority_basis.lifecycle();
        let artifact = lifecycle.declaration();
        Self {
            authority_basis,
            family: LayoutStrategyFamily::BTreeRange,
            capability: StrategyCapability::btree_range(),
            locality: StrategyLocalityProfile::OrderedPageLocality,
            amplification: StrategyAmplificationProfile::SplitMergeBounded,
            materialization: StrategyMaterializationPosture::PublishedTreeLifecycle,
            rebuild_source: StrategyRebuildSourceRequirement::PhysicalSnapshotReplay,
            corruption_isolation: StrategyCorruptionIsolationBehavior::PageScoped,
            access_lane: artifact.access_lane(),
            authority_class: artifact.authority(),
            rebuild_posture: artifact.rebuild_posture(),
            migration_posture: artifact.migration_posture(),
            projection_classes: artifact.non_authority_projection_classes(),
            canonical_key_encoding: Some(key_laws.encoding()),
            comparator_law: Some(key_laws.comparator()),
            prefix_law: key_laws.prefix(),
            range_bound_law: key_laws.range(),
            planned_counter_envelope: None,
        }
    }

    pub(super) const fn baseline_lsm_write_optimized(
        authority_basis: StrategyAuthorityBasis,
        key_laws: DeclaredKeyLawPosture,
    ) -> Self {
        let lifecycle = authority_basis.lifecycle();
        let artifact = lifecycle.declaration();
        Self {
            authority_basis,
            family: LayoutStrategyFamily::BaselineLsmWriteOptimized,
            capability: StrategyCapability::baseline_lsm_write_optimized(),
            locality: StrategyLocalityProfile::BufferedRunLocality,
            amplification: StrategyAmplificationProfile::CompactionWriteAmplified,
            materialization: StrategyMaterializationPosture::WalBufferedRunLifecycle,
            rebuild_source: StrategyRebuildSourceRequirement::WalReplay,
            corruption_isolation: StrategyCorruptionIsolationBehavior::RunScoped,
            access_lane: artifact.access_lane(),
            authority_class: artifact.authority(),
            rebuild_posture: artifact.rebuild_posture(),
            migration_posture: artifact.migration_posture(),
            projection_classes: artifact.non_authority_projection_classes(),
            canonical_key_encoding: Some(key_laws.encoding()),
            comparator_law: Some(key_laws.comparator()),
            prefix_law: None,
            range_bound_law: None,
            planned_counter_envelope: declared_strategy_counter_envelope(
                LayoutStrategyFamily::BaselineLsmWriteOptimized,
            ),
        }
    }

    pub const fn lifecycle(self) -> ArtifactFamilyLifecycleAdmission {
        self.authority_basis.lifecycle()
    }

    pub const fn key_domain(self) -> PhysicalKeyDomainWitness {
        self.authority_basis.key_domain()
    }

    pub const fn authority_basis(self) -> StrategyAuthorityBasis {
        self.authority_basis
    }

    pub const fn family(self) -> LayoutStrategyFamily {
        self.family
    }

    pub(crate) const fn capability(self) -> StrategyCapability {
        self.capability
    }

    pub(crate) const fn locality(self) -> StrategyLocalityProfile {
        self.locality
    }

    pub(crate) const fn amplification(self) -> StrategyAmplificationProfile {
        self.amplification
    }

    pub(crate) const fn materialization(self) -> StrategyMaterializationPosture {
        self.materialization
    }

    pub(crate) const fn rebuild_source(self) -> StrategyRebuildSourceRequirement {
        self.rebuild_source
    }

    pub(crate) const fn corruption_isolation(self) -> StrategyCorruptionIsolationBehavior {
        self.corruption_isolation
    }

    pub(crate) const fn access_lane(self) -> ArtifactFamilyAccessLane {
        self.access_lane
    }

    pub(crate) const fn authority_class(self) -> ArtifactFamilyAuthorityClass {
        self.authority_class
    }

    pub(crate) const fn rebuild_posture(self) -> DurableArtifactRebuildPosture {
        self.rebuild_posture
    }

    pub(crate) const fn migration_posture(self) -> DurableArtifactMigrationPosture {
        self.migration_posture
    }

    pub(crate) const fn projection_classes(self) -> &'static [DurableArtifactProjectionClass] {
        self.projection_classes
    }

    pub(crate) const fn canonical_key_encoding(self) -> Option<CanonicalKeyEncoding> {
        self.canonical_key_encoding
    }

    pub(crate) const fn comparator_law(self) -> Option<ComparatorLaw> {
        self.comparator_law
    }

    pub(crate) const fn prefix_law(self) -> Option<PrefixLawWitness> {
        self.prefix_law
    }

    pub(crate) const fn range_bound_law(self) -> Option<RangeBoundLawWitness> {
        self.range_bound_law
    }

    pub(crate) const fn planned_counter_envelope(self) -> Option<PlannedCounterEnvelope> {
        self.planned_counter_envelope
    }
}
